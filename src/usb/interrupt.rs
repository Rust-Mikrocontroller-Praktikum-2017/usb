use board::nvic::Nvic;
use board::otg_hs_global::*;
use board::otg_hs_device::*;
use collections::vec::Vec;
use collections::linked_list::LinkedList;
use alloc::boxed::Box;

static mut GLOBAL: Option<&'static mut OtgHsGlobal> = None;
static mut DEVICE: Option<&'static mut OtgHsDevice> = None;
static mut RECEIVE : Option<LinkedList<Packet>> = None;

// DEBUG
static mut PACKET_IDX : usize = 0;
static mut PACKET_HIST : [Packet; 128] = [Packet { ep: 0, data: CtlPacket::PLACEHOLDER}; 128];
static mut IRQ_IDX : usize = 0;
static mut IRQ_HIST : [(u8, u8); 128] = [(0, 0); 128];
static mut COUNT : u32 = 0u32;
static mut LAST_ROW : u16= 0;
static mut LAST_MASK : u32 = 0;
static mut GINTSTS_TRIGGERED : u32 = 0u32;
use ::render;
use stm32f7::lcd::Lcd;
static mut LCD: Option<Lcd> = None;
pub unsafe fn init_debug(lcd_: Lcd) {
	LCD = Some(lcd_);
	if let Some(ref mut lcd__)  = LCD {
		lcd__.clear_screen();
	}
}
// DEBUG END

pub unsafe fn init(global: &'static mut OtgHsGlobal, device: &'static mut OtgHsDevice, 
		nvic: &mut Nvic) {

	if let Some(ref mut lcd)  = LCD {
		render::interrupt_debug_init(lcd);
	}

	GLOBAL = Some(global);
	DEVICE = Some(device);
	RECEIVE = Some(LinkedList::new());

	if let Some(ref mut global) = GLOBAL {
		let gintsts = &mut global.otg_hs_gintsts;
		let gintmsk = &mut global.otg_hs_gintmsk;
		// Clear Gintsts to avoid interrupts before init
		gintsts.update(|_| return);

		for i in 74..78 {
			::stm32f7::interrupts::enable_interrupt(i, 1, Some(isr), nvic);
		}

		//interrupts
		gintmsk.update(|r| r.set_otgint(true));
		gintmsk.update(|r| r.set_mmism(true));

		// device interrupts
		gintmsk.update(|r| r.set_esuspm(true));
		gintmsk.update(|r| r.set_usbsuspm(true));
		gintmsk.update(|r| r.set_usbrst(true));
		gintmsk.update(|r| r.set_enumdnem(true));
		//gintmsk.update(|r| r.set_sofm(true));
		gintmsk.update(|r| r.set_oepint(true));
	}
}

unsafe fn isr(irq: u8) {
	assert!(74 <= irq && irq <= 77);
	if let Some(ref mut global) = GLOBAL {
	if let Some(ref mut device) = DEVICE {
		let mut gintsts_s = global.otg_hs_gintsts.read();
		let gintsts = gintsts_s.bits;
		let gintmsk = global.otg_hs_gintmsk.read().bits;
		GINTSTS_TRIGGERED |= gintsts;
	
		if let Some(ref mut lcd)  = LCD {
			render::interrupt_debug(gintsts, GINTSTS_TRIGGERED, 
				&mut COUNT, &mut LAST_ROW, &mut LAST_MASK, lcd);
		}

		if gintsts & !gintmsk & (1<<19) != 0{
			let a = 434;
		}

		for (i, f) in USB_ISRS.iter().enumerate().filter(|&(i, o)| o.is_some() && (gintmsk & gintsts & (1<<i) != 0)) { 
			IRQ_HIST[IRQ_IDX] = ((COUNT-1) as u8, i as u8);
			IRQ_IDX += 1;
			f.unwrap()(global, device); 
		} 
		gintsts_s.bits &= gintmsk & 0b11110000011100001111110000001010; //rw mask
		global.otg_hs_gintsts.write(gintsts_s);

	}
	}
}

type UsbIsr = Option<fn(global: &mut OtgHsGlobal, device: &mut OtgHsDevice)>;
const USB_ISRS : [UsbIsr; 32] = [
/*00*/	None,
/*01*/	Some(mmism),
/*02*/	Some(gotgint),
/*03*/	None,
/*04*/	Some(rxflvl),
/*05*/	None,
/*06*/	None,
/*07*/	None,
/*08*/	None,
/*09*/	None,
/*10*/	None,
/*11*/	None,
/*12*/	Some(usbrst),
/*13*/	Some(enumdne),
/*14*/	None,
/*15*/	None,
/*16*/	None,
/*17*/	None,
/*18*/	Some(iepint),
/*19*/	Some(oepint),
/*20*/	None,
/*21*/	None,
/*22*/	None,
/*23*/	None,
/*24*/	None,
/*25*/	None,
/*26*/	None,
/*27*/	None,
/*28*/	None,
/*29*/	None,
/*30*/	None,
/*31*/	None,
];
// Interrupt Handlers: --------------------------------------------------------
#[allow(unused_variables)]
fn mmism(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	unsafe { asm!("bkpt 0xAB"); }
}

#[allow(unused_variables)]
fn usbrst(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	//Endpoint initialization on USB reset

	//1.Set the NAK bit for all OUT endpoints
		//SNAK = 1 in OTG_DOEPCTLx (for all OUT endpoints)
	device.otg_hs_doepctl1.update(|r| r.set_snak(true));
	device.otg_hs_doepctl2.update(|r| r.set_snak(true));
	device.otg_hs_doepctl3.update(|r| r.set_snak(true));
	device.otg_hs_doepctl4.update(|r| r.set_snak(true));
	device.otg_hs_doepctl5.update(|r| r.set_snak(true));
	device.otg_hs_doepctl6.update(|r| r.set_snak(true));
	device.otg_hs_doepctl7.update(|r| r.set_snak(true));
	//2. Unmask the following interrupt bits
		//INEP0 = 1 in OTG_DAINTMSK (control 0 IN endpoint)
		//OUTEP0 = 1 in OTG_DAINTMSK (control 0 OUT endpoint)
		//STUPM = 1 in OTG_DOEPMSK
		//XFRCM = 1 in OTG_DOEPMSK
		//XFRCM = 1 in OTG_DIEPMSK
		//TOM = 1 in OTG_DIEPMSK
	device.otg_hs_daintmsk.update(|r| {
		let iepm = r.iepm();
		r.set_iepm(iepm | 0x1);
		let oepm = r.oepm();
		r.set_oepm(oepm | 0x1);
	});
	device.otg_hs_doepmsk.update(|r| {
		r.set_stupm(true);
		r.set_xfrcm(true);
	});
	device.otg_hs_diepmsk.update(|r| {
		r.set_xfrcm(true);
		r.set_tom(true);
	});
	//3. Set up the Data FIFO RAM for each of the FIFOs
		/*Program the OTG_GRXFSIZ register, to be able to receive control OUT data and 
		setup data. If thresholding is not enabled, at a minimum, this must be equal to 1 
		max packet size of control endpoint 0 + 2 Words (for the status of the control OUT 
		data packet) + 10 Words (for setup packets). */
	global.otg_hs_grxfsiz.update(|r| r.set_rxfd(0x200)); //2KB
		/*Program the OTG_DIEPTXF0 register (depending on the FIFO number chosen) to 
		be able to transmit control IN data. At a minimum, this must be equal to 1 max 
		packet size of control endpoint 0. */
	//global.otg_hs_dieptxf0.update(|r| {
	global.otg_hs_hnptxfsiz_host.update(|r| {
		//r.set_tx0fd(0x200); //default value (2KB)
		r.set_nptxfd(0x200); //default value (2KB)
		//r.set_tx0fsa(0x200); //default value 
		r.set_nptxfsa(0x200); //default value 
	});
	/*4. Program the following fields in the endpoint-specific registers for control OUT endpoint 
			0 to receive a SETUP packet */
		//STUPCNT = 3 in OTG_DOEPTSIZ0 (to receive up to 3 back-to-back SETUP packets)
	device.otg_hs_doeptsiz0.update(|r| r.set_stupcnt(3));
	/*5. For USB OTG HS in DMA mode, the OTG_DOEPDMA0 register should have a valid 	memory address 
		to store any SETUP packets received. */
	//DMA ONLY

	//At this point, all initialization required to receive SETUP packets is done.
}

#[allow(unused_variables)]
fn enumdne(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	//Endpoint initialization on enumeration completion

	/*1.On the Enumeration Done interrupt (ENUMDNE in OTG_GINTSTS), read the 
		OTG_DSTS register to determine the enumeration speed. */
	let enumspd = device.otg_hs_dsts.read().enumspd();
	//assert_eq!(enumspd, 0x3);
	if enumspd == 0x3 {
		unsafe {
			asm!("bkpt 0xAB");
		}
	}
	/*2. Program the MPSIZ field in OTG_DIEPCTL0 to set the maximum packet size. This 
		step configures control endpoint 0. The maximum packet size for a control endpoint 
		depends on the enumeration speed. */
	device.otg_hs_diepctl0.update(|r| r.set_mpsiz(0x200));
	/*3. For USB OTG HS in DMA mode, program the OTG_DOEPCTL0 register to enable 
		control OUT endpoint 0, to receive a SETUP packet. */
	//DMA ONLY?

	/*At this point, the device is ready to receive SOF packets and is configured to perform 
		control transfers on control endpoint 0. */
	global.otg_hs_gintmsk.update(|r| r.set_rxflvlm(true));
}

#[allow(unused_variables)]
fn gotgint(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	let gotgint_r = &mut global.otg_hs_gotgint;
	let gotgint_s = gotgint_r.read();
	gotgint_r.write(gotgint_s);
}

#[derive(Copy, Clone)]
struct Packet {
	ep: u8,
	data: CtlPacket, 
}
#[derive(PartialEq)]
#[derive(Copy, Clone)]
enum CtlPacket {
	Setup {
		request_type: u8,
		request: u8,
		value: u16,
		index: u16,
		length: u16,
	},
	SetupDone,
	PLACEHOLDER
}

impl Packet {
	fn new(ep: u8, count: usize, status: u8, dpid: u8, frame_no: u8, data: &[u8]) -> Packet {
		Packet { ep: ep, data: CtlPacket::new(status, count, dpid, data) }
	}
}

impl CtlPacket {
	fn new(status: u8, count: usize, dpid: u8, data: &[u8]) -> CtlPacket {
		match (status, dpid, count) {
			(0x6, 0x0, 8) => CtlPacket::Setup {
				request_type: data[0],
				request: data[1],
				value: ((data[3] as u16) << 8u16) | data[2] as u16,
				index: ((data[5] as u16) << 8u16) | data[4] as u16,
				length: ((data[7] as u16) << 8u16) | data[6] as u16,
			},
			(0x4, 0x0, 0) => {
				CtlPacket::SetupDone
			},
			_ => { unimplemented!(); }
		}
	}
}

#[allow(unused_variables)]
fn rxflvl(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	global.otg_hs_gintmsk.update(|r| r.set_rxflvlm(false));

	let grxstsp = global.otg_hs_grxstsp_host.read().bits;
	let ep = (grxstsp & 0xf) as u8;
	let count = ((grxstsp & 0x7ff0) >> 4) as usize;
	let status = ((grxstsp & (0xf << 17)) >> 17) as u8;
	let dpid = ((grxstsp >> 15) & 0x3) as u8;
	let frame_no = ((grxstsp >> 21) & 0xf) as u8;

	let mut data = Vec::<u8>::with_capacity(count as usize);
	let mut adata = [0u8; 64];
	let ptr = (0x1000usize + 0x4004_0000) as *const u32;

	let mut read_bytes = count;
	for i in 0..(count+3)/4 {
		unsafe {
			let word = ::core::ptr::read_volatile(ptr);
			for j in 0..::core::cmp::min(4, read_bytes) {
				data.push((word >> (j*8)) as u8);
				adata[i*4+j] = data[i*4+j];
			}
			read_bytes -= 4;
		}
	}
	
	let packet = Packet::new(ep, count, status, dpid, frame_no, &data);
	unsafe {
		PACKET_HIST[PACKET_IDX] = packet; 
		PACKET_IDX += 1;
	}
	unsafe {
		if let Some(ref mut list) = RECEIVE {
			list.push_back(packet);
		}
	}

	global.otg_hs_gintmsk.update(|r| r.set_rxflvlm(true));
}

fn send(data: &[u32], byte_cnt: usize, device: &mut OtgHsDevice, global: &mut OtgHsGlobal) {
	assert!(byte_cnt < 64); // < MPS
	unsafe{
	//if debug_1 { return; }
	debug_1 = true;
	}

	let word_cnt = data.len();
	device.otg_hs_dieptsiz0.update(|r| {
		r.set_pktcnt(1); 
		r.set_xfrsiz(byte_cnt as u8);
	});
	device.otg_hs_diepctl0.update(|r| {
		r.set_epena(true);
		r.set_cnak(true);
	});
	while device.otg_hs_dtxfsts0.read().ineptfsav() < word_cnt as u16 {}
	for i in 0..word_cnt {
			let ptr = (0x1000usize + 0x4004_0000) as *mut u32;
			unsafe { ::core::ptr::write_volatile(ptr, data[i]); }
	}
	while device.otg_hs_dieptsiz0.read().xfrsiz() > 0 {}
	device.otg_hs_diepempmsk.update(|r| { let a = r.ineptxfem(); r.set_ineptxfem(a | 0x1); });
	global.otg_hs_gintmsk.update(|r| r.set_iepint(true));
}

static mut debug_1 : bool = false;

#[allow(unused_variables)]
fn iepint(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	let iepint = device.otg_hs_daint.read().iepint();
	if iepint & 0x1 == 1 {
		let int0 = device.otg_hs_diepint0.read();
		if int0.txfe() {
			device.otg_hs_diepempmsk.update(|r| { let a = r.ineptxfem(); r.set_ineptxfem(a & !(0x1)); });
		}
		if int0.xfrc() {
			unsafe { debug_1 = false; }
		}
		device.otg_hs_diepint0.write(int0);
	}
}

#[allow(unused_variables)]
fn oepint(global: &mut OtgHsGlobal, device: &mut OtgHsDevice) {
	let oepint = device.otg_hs_daint.read().oepint();

	unsafe {
		if PACKET_IDX > 1 {
			let last = PACKET_HIST[PACKET_IDX-1];
			let blast = PACKET_HIST[PACKET_IDX-2];
			if let CtlPacket::Setup { request, .. } = blast.data {
				if let CtlPacket::SetupDone {} = last.data {
					if request == 5 {
						let a = 5;
					}
				}
			}
		}
	}
	
	//endpoint
	if oepint & 0x1 == 1 {
		if device.otg_hs_doepint0.read().stup()	{
			device.otg_hs_doepint0.update(|r| r.set_stup(true));
			let stupcnt = device.otg_hs_doeptsiz0.read().stupcnt();
			assert!(stupcnt <= 3);
			unsafe {
			if let Some(ref mut list) = RECEIVE {
				let mut last_packet: Option<Packet> = None;
				let mut done = false;
				while let Some(packet) = list.pop_front() {
					match packet.data { 
						CtlPacket::Setup {..} 
							=> last_packet = Some(packet), 
						CtlPacket::SetupDone {..}
							=> { done = true; break },
						_ => panic!()
					}
				}
				if !done && last_packet.is_some() {
					list.push_front(last_packet.unwrap());
					return;
				}
				
				if let Some(CtlPacket::Setup {request, value, ..} ) = last_packet.map(|x| x.data) {
					// GET DESCRIPTOR
					if request == 6 {
						let desc_type = (value >> 8) & 0xf;
						let desc_idx = value & 0xf;
						// DEVICE Descriptor
						if desc_type == 1 {
								let length 		: u8	= 18;
								let desc_type	: u8 	= desc_type as u8;
								let bcd_usb		: u16 	= 0x0200;
								let class		: u8	= 0; //interface specific
								let subclass	: u8	= 0;
								let proto		: u8	= 0;
								let mps			: u8	= 64;
								let vendor		: u16	= 0x3412;
								let product		: u16	= 0x7856;
								let bcd_device	: u16	= 0x5713;
								let ivendor		: u8	= 0x0;
								let iproduct	: u8	= 0x0;
								let iserial		: u8	= 0x0;
								let numconfig	: u8	= 1; //num configuration descriptors

								#[repr(C)]
								struct dev_desc(u8, u8, u16, u8, u8, u8, u8, u16, u16, u16, u8, u8, u8, u8, u16);
								assert_eq!(::core::mem::size_of::<dev_desc>(), 20);
								let device_descriptor = dev_desc(length, desc_type, bcd_usb, class, subclass,
									proto, mps, vendor, product, bcd_device, ivendor, iproduct, iserial, 
									numconfig, 0);
								let data = ::core::intrinsics::transmute::<dev_desc, [u32; 5]>(device_descriptor);
								send(&data, 5, device, global);

						}
					}
					// SET ADDRESS
					else if request == 5 {
						let a = 5;
					}
				}

				let a = 5;
				let x = 19;
				}
			}
			device.otg_hs_doeptsiz0.update(|r| r.set_stupcnt(3));
		}
	}
}
