use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::RAM;
use crate::GB::memory::registers::LCDC;
use crate::GB::PPU::PPU;
use crate::GB::memory::UseMemory;

macro_rules! create_memory {
    () => {
        Rc::new(RefCell::new(RAM::new()))
    };
}

macro_rules! test_get_bit_flag_function {
    ($func: ident, $get_func: ident, $register: ident, $mask: expr) => {
        #[test]
        fn $func() {
            let mem = create_memory!();
            mem.borrow_mut().write($register, $mask as u8);
            let ppu = PPU::new(Rc::clone(&mem));
            assert_eq!(ppu.$get_func(), true);
            mem.borrow_mut().write($register, 0);
            assert_eq!(ppu.$get_func(), false);
        }
    };
}

macro_rules! test_set_bit_flag_function {
    ($func: ident, $set_func: ident, $register: ident, $mask: expr) => {
        #[test]
        fn $func() {
            let mem = create_memory!();
            mem.borrow_mut().write($register, $mask as u8);
            let mut ppu = PPU::new(Rc::clone(&mem));
            ppu.$set_func(false);
            assert_eq!((mem.borrow().read($register) & $mask) == 0, true);
            ppu.$set_func(true);
            assert_eq!((mem.borrow().read($register) & $mask) != 0, true);
        }
    };
}

macro_rules! test_get_set_bit_flag_function {
    ($func_get: ident, $func_set: ident, $ppu_get_func: ident, $ppu_set_func: ident, $register: ident, $mask: expr) => {
        test_get_bit_flag_function!($func_get, $ppu_get_func, $register, $mask);
        test_set_bit_flag_function!($func_set, $ppu_set_func, $register, $mask);
    };
}

#[test]
fn test_new_ppu() {
    let mut mem = create_memory!();
    mem.borrow_mut().write(LCDC, 0xFE);
    let ppu = PPU::new(Rc::clone(&mem));
    assert_eq!(ppu.read_memory(LCDC), 0xFE);
    mem.borrow_mut().write(LCDC, 0xFD);
    assert_eq!(ppu.read_memory(LCDC), 0xFD);
}

test_get_set_bit_flag_function!(test_ppu_get_bg_win_enabled_flag, test_ppu_set_bg_win_enabled_flag,
    get_bg_win_enabled_flag, set_bg_win_enabled_flag, LCDC, 0b0000_0001);
test_get_set_bit_flag_function!(test_ppu_get_obj_enabled_flag, test_ppu_set_obj_enabled_flag,
    get_obj_enabled_flag, set_obj_enabled_flag, LCDC, 0b0000_0010);
test_get_set_bit_flag_function!(test_ppu_get_obj_size_flag, test_ppu_set_obj_size_flag,
    get_obj_size_flag, set_obj_size_flag, LCDC, 0b0000_0100);
test_get_set_bit_flag_function!(test_ppu_get_bg_tile_map_area_flag, test_ppu_set_bg_tile_map_area_flag,
    get_bg_tile_map_area_flag, set_bg_tile_map_area_flag, LCDC, 0b0000_1000);
test_get_set_bit_flag_function!(test_ppu_get_bg_win_tiles_area_flag, test_ppu_set_bg_win_tiles_area_flag,
    get_bg_win_tiles_area_flag, set_bg_win_tiles_area_flag, LCDC, 0b0001_0000);
test_get_set_bit_flag_function!(test_ppu_get_win_enabled_flag, test_ppu_set_win_enabled_flag,
    get_win_enabled_flag, set_win_enabled_flag, LCDC, 0b0010_0000);
test_get_set_bit_flag_function!(test_ppu_get_win_tile_map_area_flag, test_ppu_set_win_tile_map_area_flag,
    get_win_tile_map_area_flag, set_win_tile_map_area_flag, LCDC, 0b0100_0000);
test_get_set_bit_flag_function!(test_ppu_get_lcd_enabled_flag, test_ppu_set_lcd_enabled_flag,
    get_lcd_enabled_flag, set_lcd_enabled_flag, LCDC, 0b1000_0000);
