#![no_std]
#![no_main]
#![feature(sort_floats)]

// extern crate alloc;

pub mod mag_calibration;

use ahrs::{Ahrs, Madgwick};
use core::{cell::RefCell, fmt::Write};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    i2c, interrupt,
    peripherals::{self, Peripherals, TIMG0},
    prelude::*,
    timer::{Timer, Timer0, TimerGroup},
    Cpu, UsbSerialJtag, IO,
};
use icm42670::{accelerometer::Accelerometer, Address, Icm42670, PowerMode as ImuPowerMode};
use nalgebra::{Quaternion, Unit, Vector3};
use shared_bus::BusManagerSimple;

static USB_SERIAL: Mutex<RefCell<Option<UsbSerialJtag>>> = Mutex::new(RefCell::new(None));

static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>>>>> = Mutex::new(RefCell::new(None));
static QUAT: Mutex<RefCell<Option<Unit<Quaternion<f64>>>>> = Mutex::new(RefCell::new(None));
static FFT_X: Mutex<RefCell<Option<u32>>> = Mutex::new(RefCell::new(None));

const FFT_SAMPLE_COUNT: usize = 512;

#[entry]
fn main() -> ! {
    esp_alloc::heap_allocator!(32 * 1024);
    println!("Initialising hadeda");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    interrupt::enable(
        peripherals::Interrupt::TG0_T0_LEVEL,
        interrupt::Priority::Priority1,
    )
    .unwrap();
    timer0.start(10u64.millis());
    timer0.listen();

    let mut usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);

    usb_serial.listen_rx_packet_recv_interrupt();

    critical_section::with(|cs| {
        TIMER0.borrow_ref_mut(cs).replace(timer0);
        USB_SERIAL.borrow_ref_mut(cs).replace(usb_serial);
    });

    interrupt::enable(
        peripherals::Interrupt::USB_DEVICE,
        interrupt::Priority::Priority1,
    )
    .unwrap();

    interrupt::set_kind(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt1,
        interrupt::InterruptKind::Edge,
    );

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let sda = io.pins.gpio10;
    let scl = io.pins.gpio8;
    let i2c = i2c::I2C::new(peripherals.I2C0, sda, scl, 400u32.kHz(), &clocks);

    let bus = BusManagerSimple::new(i2c);

    let proxy_1 = bus.acquire_i2c();

    let mut imu = Icm42670::new(proxy_1, Address::Primary).unwrap();

    let device_id = imu.device_id().unwrap();
    println!("Device ID ICM42670p: {:#02x}", device_id);

    imu.set_power_mode(ImuPowerMode::SixAxisLowNoise).unwrap();

    let mut ahrs = Madgwick::default();

    let mut samples = [0f32; FFT_SAMPLE_COUNT];
    let mut amplitudes = [0u32; FFT_SAMPLE_COUNT / 2];
    let mut sample_index = 0;

    loop {
        let gyro_data = imu.gyro_norm().unwrap();
        let accel_data = imu.accel_norm().unwrap();

        let gyroscope = Vector3::new(gyro_data.x as f64, gyro_data.y as f64, gyro_data.z as f64)
            * (core::f64::consts::PI / 180.0);
        let accelerometer = Vector3::new(
            accel_data.x as f64,
            accel_data.y as f64,
            accel_data.z as f64,
        );

        let quat = ahrs.update_imu(&gyroscope, &accelerometer).unwrap();

        samples[sample_index] = quat.euler_angles().0 as f32;

        critical_section::with(|cs| {
            if sample_index == FFT_SAMPLE_COUNT - 1 {
                let spectrum = microfft::real::rfft_512(&mut samples);
                // since the real-valued coefficient at the Nyquist frequency is packed into the
                // imaginary part of the DC bin, it must be cleared before computing the amplitudes
                spectrum[0].im = 0.0;
                spectrum
                    .iter()
                    .enumerate()
                    .for_each(|(i, c)| amplitudes[i] = c.norm_sqr() as u32);
            }
            FFT_X
                .borrow_ref_mut(cs)
                .replace(amplitudes[sample_index % FFT_SAMPLE_COUNT / 2]);
            QUAT.borrow_ref_mut(cs).replace(*quat);
        });

        sample_index = (sample_index + 1) % FFT_SAMPLE_COUNT;
    }
}

#[interrupt]
fn TG0_T0_LEVEL() {
    critical_section::with(|cs| {
        let quat = QUAT.borrow_ref(cs);
        let fft_x = FFT_X.borrow_ref(cs);
        if quat.is_some() {
            let (roll, pitch, yaw) = quat.unwrap().euler_angles();
            writeln!(
                USB_SERIAL.borrow_ref_mut(cs).as_mut().unwrap(),
                "/*{},{},{},{}*/",
                roll * 180.0 / core::f64::consts::PI,
                pitch * 180.0 / core::f64::consts::PI,
                yaw * 180.0 / core::f64::consts::PI,
                fft_x.unwrap_or(0),
            )
            .ok();
        }

        let mut timer0 = TIMER0.borrow_ref_mut(cs);
        let timer0 = timer0.as_mut().unwrap();

        timer0.clear_interrupt();
        timer0.start(10u64.millis());
    });
}

#[interrupt]
fn USB_DEVICE() {
    critical_section::with(|cs| {
        let mut usb_serial = USB_SERIAL.borrow_ref_mut(cs);
        let usb_serial = usb_serial.as_mut().unwrap();
        writeln!(usb_serial, "USB serial interrupt").unwrap();
        while let nb::Result::Ok(c) = usb_serial.read_byte() {
            writeln!(usb_serial, "Read byte: {:02x}", c).unwrap();
        }
        usb_serial.reset_rx_packet_recv_interrupt();
    });
}
