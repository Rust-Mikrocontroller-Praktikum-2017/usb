#![no_std]
#![no_main]
#![feature(asm)]
#[allow(unused_variables)]

extern crate stm32f7_discovery as stm32f7;
extern crate embedded_stm32f7;

// initialization routines for .data and .bss
extern crate r0;
extern crate cortex_m;

use stm32f7::{system_clock, board};
use embedded_stm32f7::otg_hs_global::{OtgHsGintsts};

#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
	extern "C" {
		static __DATA_LOAD: u32;
		static __DATA_END: u32;
		static mut __DATA_START: u32;
		static mut __BSS_START: u32;
		static mut __BSS_END: u32;
	}

	let data_load = &__DATA_LOAD;
	let data_start = &mut __DATA_START;
	let data_end = &__DATA_END;
	let bss_start = &mut __BSS_START;
	let bss_end = &__BSS_END;

	// initializes the .data section
	//(copy the data segment initializers from flash to RAM)
	r0::init_data(data_start, data_end, data_load);
	// zeroes the .bss section
	r0::zero_bss(bss_start, bss_end);
	let scb = stm32f7::cortex_m::peripheral::scb_mut();
	scb.cpacr.modify(|v| v | 0b1111 << 20);

	main(board::hw());
}

fn main(hw: board::Hardware) -> ! {
	let board::Hardware { rcc,
		pwr,
		flash,
		fmc,
		ltdc,
		gpio_a,
		gpio_b,
		gpio_c,
		gpio_d,
		gpio_e,
		gpio_f,
		gpio_g,
		gpio_h,
		gpio_i,
		gpio_j,
		gpio_k,
		i2c_3,
		sai_2,
		syscfg,
		ethernet_mac,
		ethernet_dma,
		otg_hs_global,
		otg_hs_device,
		nvic,
		.. } = hw;

	system_clock::init(rcc, pwr, flash);   
	rcc.ahb1enr.update(|r| r.set_otghsen(true));
	rcc.ahb1enr.update(|r| r.set_otghsulpien(true));

	let ictr: u8 = nvic.ictr.read().intlinesnum();
	let ictr_addr = &nvic.ictr;

	let iser0: u32 = nvic.iser0.read().setena();
	let iser1: u32 = nvic.iser1.read().setena();
	let iser2: u32 = nvic.iser2.read().setena();
	nvic.iser0.update(|r| r.set_setena(!0));
	nvic.iser1.update(|r| r.set_setena(!0));
	nvic.iser2.update(|r| r.set_setena(!0));
	let iser0_addr = &nvic.iser0;
	let _iser0: u32 = nvic.iser0.read().setena();
	let _iser1: u32 = nvic.iser1.read().setena();
	let _iser2: u32 = nvic.iser2.read().setena();
	let iabr0 = nvic.iabr0.read().active();
	let iabr1 = nvic.iabr1.read().active();
	let iabr2 = nvic.iabr2.read().active();
	nvic.icpr0.update(|r| r.set_clrpend(!0));
	nvic.icpr1.update(|r| r.set_clrpend(!0));
	nvic.icpr2.update(|r| r.set_clrpend(!0));
	nvic.ipr19.update(|r| r.set_ipr_n1(1)); // set priority of irq77
	//unsafe { self.iser[usize::from(nr / 32)].write(1 << (nr % 32)) }
	
	//fn abc () {
	//}

	//unsafe { stm32f7::interrupts::HANDLE_INT = Some(abc); }
	let mut stir : embedded_stm32f7::nvic::Stir = embedded_stm32f7::nvic::Stir::default();
	stir.set_intid(76);
	nvic.stir.write(stir);

	// Clear Gintsts to avoid interrupts before init
	otg_hs_global.otg_hs_gintsts.update(|_| return);

	//core init

	otg_hs_global.otg_hs_gccfg.update(|r| r.set_pwrdwn(false));

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_physel(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_tsdps(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpifsls(false));

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpievbusd(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpievbusi(false));
	
	while ! otg_hs_global.otg_hs_grstctl.read().ahbidl() {};
	otg_hs_global.otg_hs_grstctl.update(|r| r.set_csrst(true));
	while otg_hs_global.otg_hs_grstctl.read().csrst() {};

	//options

	otg_hs_global.otg_hs_gahbcfg.update(|r| r.set_gint(true));
	otg_hs_global.otg_hs_gahbcfg.update(|r| r.set_ptxfelvl(true)); //completely empty

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_hnpcap(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_srpcap(false));
	//otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_tocal()); //not necessary for hs?
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_trdt(0x9)); //only valid value for hs?

	//interrupts
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_otgint(true));
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_mmism(true));

	// Wait till we enter device mode
	loop
	{
		let cmod = otg_hs_global.otg_hs_gintsts.read().cmod();
		if !cmod {
			break;
		}
		//sleep
	}

	otg_hs_global.otg_hs_gccfg.update(|r| r.set_vbden(true));
	//otg_hs_global.otg_hs_pcgcctl.write(embedded_stm32f7::otg_hs_global::OtgHsPcgcctl::default());
	//device mode init
	//options
	otg_hs_device.otg_hs_dcfg.update(|r| r.set_dspd(0)); // high speed
	otg_hs_device.otg_hs_dcfg.update(|r| r.set_nzlsohsk(false)); //no clue
	// interrupts
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_esuspm(true));
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_usbsuspm(true));
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_usbrst(true));
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_enumdnem(true));
	otg_hs_global.otg_hs_gintmsk.update(|r| r.set_sofm(true));

	//let mut full = embedded_stm32f7::otg_hs_global::OtgHsGintmsk::default();
	//let mask = 0b11111000101111001111110011011110;
	//full.bits = mask;
	//otg_hs_global.otg_hs_gintmsk.write(full);

	loop {
	}

	// Manually poll interrupts (GINTSTS)
	let mut int : [OtgHsGintsts; 10] = [OtgHsGintsts::default(); 10];
	let mut i = 0usize;
	while i < 10usize {
		let now = otg_hs_global.otg_hs_gintsts.read();
		otg_hs_global.otg_hs_gintsts.update(|_| return);
		
		if now != int[i] {
			int[i] = now;
			i += 1;
		}
	}
	loop {
	}
}
