extern crate stm32f7_discovery as stm32f7;
use stm32f7::lcd;


pub const TEST_DIM : (u16, u16) = (14, 8);

type Segment = [[usize; 2]; 2];
const TOP   : Segment = [[0, 1], [0,2]];
const MID   : Segment = [[3, 1], [3,2]];
const BOT   : Segment = [[6, 1], [6,2]];
const TOP_L : Segment = [[1, 0], [2,0]];
const TOP_R : Segment = [[1, 3], [2,3]];
const BOT_L : Segment = [[4, 0], [5,0]];
const BOT_R : Segment = [[4, 3], [5,3]];

const DIGIT_TO_SEGMENTS : [&[Segment]; 17] = [
	&[TOP, TOP_R, TOP_L,      BOT_L, BOT_R, BOT], //0
	&[     TOP_R,                    BOT_R     ], //1
	&[TOP, TOP_R,        MID, BOT_L,        BOT], //2
	&[TOP, TOP_R,        MID,        BOT_R, BOT], //3
	&[     TOP_R, TOP_L, MID,        BOT_R     ], //4
	&[TOP,        TOP_L, MID,        BOT_R, BOT], //5
	&[TOP,        TOP_L, MID, BOT_L, BOT_R, BOT], //6
	&[TOP, TOP_R,                    BOT_R     ], //7
	&[TOP, TOP_R, TOP_L, MID, BOT_L, BOT_R, BOT], //8
	&[TOP, TOP_R, TOP_L, MID,        BOT_R, BOT], //9
	&[TOP, TOP_R, TOP_L, MID, BOT_L, BOT_R     ], //A
	&[            TOP_L, MID, BOT_L, BOT_R, BOT], //b
	&[                   MID, BOT_L,        BOT], //c
	&[     TOP_R,        MID, BOT_L, BOT_R, BOT], //d
	&[TOP,        TOP_L, MID, BOT_L,        BOT], //E
	&[TOP,        TOP_L, MID, BOT_L,           ], //F
	&[                   MID,                  ], //-
];
pub fn render_number(num: i64, base: u32, dim: (u16, u16), offset: (u16, u16), digits: u8, lcd: &mut lcd::Lcd) {
	render_number_color(num, base, dim, offset, digits, 0xffff, lcd);
}

pub fn render_number_color(mut num: i64, mut base: u32, dim: (u16, u16), mut offset: (u16, u16), digits: u8, color: u16, lcd: &mut lcd::Lcd) {
	fn render_digit(digit: u32, (height, width): (u16, u16), offset: (u16, u16), color: u16, lcd: &mut lcd::Lcd) {
		let segments = DIGIT_TO_SEGMENTS[digit as usize];
		let mut buffer = [[false; 4]; 7];
		for segment in segments {
			for sub_segment in segment {
				buffer[sub_segment[0]][sub_segment[1]] = true;
			}
		}
		for x in 0..width {
			for y in 0..height {
				let color = 
					if !buffer[(y*7 / height) as usize][(x*4 / width) as usize] { 0x0000 }
					else { color };				

				lcd.print_point_color_at(x + offset.1 - width, y + offset.0, color);
			}
		}
	}
	let neg = num < 0 && base > 1;
	if neg {num *= -1;}
	if base == 1 { base = 2; }
	let mut num = num as u64;
	let mut digit_i = 0;
	while digit_i < digits {
		let digit = num % (base as u64);
		num /= base as u64;
		render_digit(digit as u32, dim, offset, color, lcd);
		offset.1 -= 5*dim.1/4;

		digit_i += 1;
		//if num == 0 { break; }
	}
	if neg {
		render_digit(digits as u32, dim, offset, color, lcd);
		// digit_i += 1;  //uncomment when continuing to implement this code
	}

}


pub fn interrupt_debug(gintsts: u32, gotint: u32, gintsts_triggered: u32, count: &mut u32, last_row: &mut u16, last_mask: &mut u32, lcd: &mut lcd::Lcd) {
	//COUNT
	render_number(*count as i64, 10, TEST_DIM, (0, 400), 4, lcd);
	*count += 1;

	//TRIGGERED
	render_number(gintsts_triggered as i64, 2, TEST_DIM, (30, 400), 32, lcd);
	//GINTSTS
	render_number(gintsts as i64, 2, TEST_DIM, (100, 400), 32, lcd);
	//GOTINT
	if gintsts & 0x4 != 0 {
		render_number(gotint as i64, 2, TEST_DIM, (120, 400), 32, lcd);
	}
	let mut row = 0;
	//IRQ Column
	for i in 0i64..32 {
		if (gintsts & (1<<i)) != 0 {
			let color = if (*last_mask & gintsts) & (1<<i) != 0 {0xffffu16} 
				else {0xf0f0u16};
			render_number_color(i, 10, TEST_DIM, 
				(15+row*15, 460), 2, color, lcd);
			row += 1;
		}
	}
	//CLEAR IRQ Column Remainder Efficient
	for i in row..*last_row {
		for x in 0..(9*TEST_DIM.1)/4+1 {
			for y in 0..TEST_DIM.0 {
				lcd.print_point_color_at(460-x, 15+i*15+y, 0);
			}
		}
	}
	*last_row = row;
	*last_mask = gintsts;
}

pub fn interrupt_debug_init(lcd: &mut lcd::Lcd) {
	lcd.clear_screen();
	render_number(5i64, 10, TEST_DIM, (100, 200), 1, lcd);
	for i in 0..32 {
		let j = i % 10;
		let k = i / 10;
		if j == 0 {
			render_number(k as i64, 10, TEST_DIM, (65, 400-(i*5*TEST_DIM.1)/4), 1, lcd);
		}
		render_number(j as i64, 10, TEST_DIM, (80, 400-(i*5*TEST_DIM.1)/4), 1, lcd);
	}
}
