#![no_std]
#![no_main]

use defmt::{error, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{UART1, USB};
use embassy_rp::uart::{Config as UartConfig, Uart};
use embassy_rp::usb::Driver;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::State;
use embassy_usb::{Builder, Config as UsbConfig};
use static_cell::StaticCell;
use uart_to_gamepad::{
    configure_usb_hid, GamepadState, InputSource, OutputSink, UartInputSource, UsbHidOutput,
};

#[cfg(feature = "dev-panic")]
use panic_probe as _;
#[cfg(feature = "prod-panic")]
use panic_reset as _;

bind_interrupts!(struct Irqs {
    UART1_IRQ => embassy_rp::uart::InterruptHandler<UART1>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

/// Signal for passing gamepad state from input to output task.
/// Using Signal instead of Channel provides "latest value wins" semantics,
/// which is appropriate for gamepad state where we only care about the most recent input.
static STATE_SIGNAL: StaticCell<Signal<CriticalSectionRawMutex, GamepadState>> = StaticCell::new();

/// USB device configuration buffer.
static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

/// HID state.
static HID_STATE: StaticCell<State> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("UART-to-Gamepad starting...");

    let p = embassy_rp::init(embassy_rp::config::Config::default());

    // Initialize the gamepad state signal (latest-value semantics)
    let signal = STATE_SIGNAL.init(Signal::new());

    // --- UART Setup ---
    let mut uart_config = UartConfig::default();
    uart_config.baudrate = 115_200;

    let uart = Uart::new(
        p.UART1,
        p.PIN_8, // TX
        p.PIN_9, // RX
        Irqs,
        p.DMA_CH0,
        p.DMA_CH1,
        uart_config,
    );
    let (_tx, rx) = uart.split();
    let uart_input = UartInputSource::new(rx);

    // --- USB Setup ---
    let usb_driver = Driver::new(p.USB, Irqs);

    let mut usb_config = UsbConfig::new(0x1209, 0x0001); // pid.codes test VID/PID
    usb_config.manufacturer = Some("Rust Gamepad");
    usb_config.product = Some("UART-to-Gamepad Bridge");
    usb_config.serial_number = Some("001");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    let config_descriptor = CONFIG_DESCRIPTOR.init([0; 256]);
    let bos_descriptor = BOS_DESCRIPTOR.init([0; 256]);
    let msos_descriptor = MSOS_DESCRIPTOR.init([0; 256]);
    let control_buf = CONTROL_BUF.init([0; 64]);

    let mut builder = Builder::new(
        usb_driver,
        usb_config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    // Configure HID class
    let hid_state = HID_STATE.init(State::new());
    let hid_writer = configure_usb_hid(&mut builder, hid_state);

    // Build the USB device
    let usb_device = builder.build();

    // Create output
    let usb_output = UsbHidOutput::new(hid_writer);

    // Optional: LED for error indication (on-board LED on Pico)
    let led = Output::new(p.PIN_25, Level::Low);

    // Spawn tasks (unwrap the SpawnToken, then spawn)
    spawner.spawn(usb_task(usb_device).unwrap());
    spawner.spawn(input_task(uart_input, signal, led).unwrap());
    spawner.spawn(output_task(usb_output, signal).unwrap());

    info!("UART-to-Gamepad initialized, waiting for data...");
}

/// USB device task - runs the USB stack.
#[embassy_executor::task]
async fn usb_task(mut device: embassy_usb::UsbDevice<'static, Driver<'static, USB>>) {
    device.run().await;
}

/// Input task - reads from UART and signals the latest gamepad state.
#[embassy_executor::task]
async fn input_task(
    mut input: UartInputSource<'static>,
    signal: &'static Signal<CriticalSectionRawMutex, GamepadState>,
    mut led: Output<'static>,
) {
    loop {
        match input.receive().await {
            Ok(state) => {
                // Signal the latest gamepad state (overwrites any pending value)
                signal.signal(state);
            }
            Err(e) => {
                error!("Input error: {:?}", e);
                // Signal neutral state on error to prevent stale inputs
                signal.signal(GamepadState::neutral());
                // Toggle LED to indicate error
                led.toggle();
            }
        }
    }
}

/// Output task - waits for gamepad state signals and sends to USB HID.
#[embassy_executor::task]
async fn output_task(
    mut output: UsbHidOutput<'static>,
    signal: &'static Signal<CriticalSectionRawMutex, GamepadState>,
) {
    // Wait for USB to be ready
    output.wait_ready().await;
    info!("USB HID ready, forwarding gamepad state...");

    loop {
        // Wait for the next gamepad state (blocks until signaled)
        let state = signal.wait().await;
        if let Err(e) = output.send(&state).await {
            error!("Output error: {:?}", e);
        }
    }
}
