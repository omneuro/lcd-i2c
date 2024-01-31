#![no_std]
#![no_main]

pub mod stm32_lib;

use cortex_m_rt::entry;
use stm32_lib::rcc;
use stm32f4::stm32f446;

const SLAVE_ADDRESS: u8 = 0x4e;

fn i2c_config(
    rcc: &mut stm32f4::stm32f446::RCC,
    gpio: &mut stm32f4::stm32f446::GPIOB,
    i2c: &mut stm32f4::stm32f446::I2C1,
) {
    //Enable 12c and gpio clocks
    rcc.apb1enr.write(|w| w.i2c1en().set_bit());
    rcc.ahb1enr.write(|w| w.gpioben().set_bit());

    //configure gpio pins

    gpio.moder.write(|w| w.moder8().bits(0b10)); // place pin PB8 and PB9 in alternate fucntion mode
    gpio.moder.modify(|_r, w| w.moder9().bits(0b10));

    gpio.otyper.modify(|_r, w| w.ot8().set_bit()); // enabling open drain mode for pin PB8 and PB9
    gpio.otyper.modify(|_r, w| w.ot9().set_bit());

    gpio.ospeedr.modify(|_r, w| w.ospeedr8().bits(0b11)); // setting the pins PB8 and PB9 to high speed (fastest)
    gpio.ospeedr.modify(|_r, w| w.ospeedr9().bits(0b11));

    gpio.pupdr.modify(|_r, w| unsafe { w.pupdr8().bits(0b01) }); //setting the pull up resistors for pinis PB8 and PB9
    gpio.pupdr.modify(|_r, w| unsafe { w.pupdr9().bits(0b01) });

    gpio.afrh.modify(|_r, w| w.afrh8().bits(0b0100)); //setting the alternate mode to i2c for pins PB8 and PB9
    gpio.afrh.modify(|_r, w| w.afrh9().bits(0b0100));

    i2c.cr1.write(|w| w.swrst().set_bit()); // put i2c in the reset state
    i2c.cr1.modify(|_r, w| w.swrst().clear_bit()); //take i2c out of reset state

    // Program the peripheral input clock in I2c_cr2 register in order to generate correct timings
    i2c.cr2.write(|w| unsafe { w.freq().bits(0b101101) }); //setting periphercal input clock fequency to 45mhz (current max value of APB)
    i2c.cr2.modify(|_r, w| w.itevten().enabled()); //enable even interrupts

    //configure the clock control registers
    i2c.ccr.write(|w| w.f_s().standard());
    i2c.ccr.modify(|_r, w| w.duty().duty2_1());
    i2c.ccr.modify(|_r, w| unsafe { w.ccr().bits(0b11100001) }); //setting crr to 225 (calculated see manual)
    i2c.trise.modify(|_r, w| w.trise().bits(0b101110)); // configure the rise time register (calculated see manual)
    i2c.cr1.modify(|_r, w| w.pe().set_bit()); // Enable the peripheral in i2c_cr1 register
}
fn timer_config(rcc: &mut stm32f4::stm32f446::RCC, timer: &mut stm32f4::stm32f446::TIM11) {
    //Enable Timer clock
    rcc.apb2enr.modify(|_r, w| w.tim11en().set_bit());

    //initialize timer for delay
    // Set the prescaler and the ARR
    timer.psc.modify(|_r, w| w.psc().bits(0b0000000010110011)); //180MHz/180 = 1MHz ~ 1us, prescalar set to 179, ie. 179+1 = 180;
    timer.arr.modify(|_r, w| unsafe { w.arr().bits(0xffff) });

    //Enable the Timer, and wait for the update Flag to set
    timer.cr1.modify(|_r, w| w.cen().set_bit());
    timer.sr.read().uif().bit();
    while !timer.sr.read().uif().bit() {}
}

pub fn i2c_start(i2c: &mut stm32f446::I2C1) {
    //send the Start condition
    i2c.cr1.modify(|_r, w| w.ack().set_bit()); //enabling the acknowledgement bit
    i2c.cr1.modify(|_r, w| w.start().set_bit());
    while !i2c.sr1.read().sb().is_start() { // waiting for the start condition
    }
}

fn i2c_address(address: u8, i2c: &mut stm32f446::I2C1) {
    //send the slave address to the DR register
    i2c.dr.write(|w| w.dr().bits(address));
    while !i2c.sr1.read().addr().is_match() {}
    let _temp = i2c.sr1.read(); //reading sr1 and sr2 to clear the addr bit
    let _temp2 = i2c.sr2.read();
}

#[entry]
fn main() -> ! {
    let peripherals = stm32f446::Peripherals::take().unwrap();
    let mut rcc = peripherals.RCC;
    let mut timer = peripherals.TIM11; //used for the delay
    let mut flash = peripherals.FLASH;
    let mut gpio = peripherals.GPIOB; //general purpose pin normally used as output.
    let mut pwr = peripherals.PWR;
    let mut i2c = peripherals.I2C1;

    rcc::initialize_clock(&mut rcc, &mut pwr, &mut flash);
    timer_config(&mut rcc, &mut timer);
    i2c_config(&mut rcc, &mut gpio, &mut i2c);
    i2c_start(&mut i2c);
    i2c_address(SLAVE_ADDRESS, &mut i2c);

    loop {
        // your code goes here
    }
}
