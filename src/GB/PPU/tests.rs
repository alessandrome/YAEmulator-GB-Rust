use std::cell::RefCell;
use std::rc::Rc;
use crate::GB::memory::{RAM, VRAM_BLOCK_0_ADDRESS, VRAM_BLOCK_1_ADDRESS, VRAM_BLOCK_2_ADDRESS};
use crate::GB::memory::registers::LCDC;
use crate::GB::PPU::PPU;
use crate::GB::memory::UseMemory;
use crate::GB::PPU::tile::TILE_SIZE;

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

#[test]
fn test_ppu_get_tile() {
    let mut mem = create_memory!();
    let test_sprite: [u8; TILE_SIZE] = [
        0x00, 0x3C,
        0x3C, 0x66,
        0x5A, 0xDB,
        0x5A, 0x81,
        0x7E, 0x99,
        0x5A, 0xDB,
        0x3C, 0x7E,
        0x00, 0x3C,
    ];
    let tile_id: u8 = 3;
    let tile_address = VRAM_BLOCK_0_ADDRESS + TILE_SIZE * tile_id as usize;
    for i in 0..TILE_SIZE {
        mem.borrow_mut().write((tile_address + i) as u16, test_sprite[i]);
    }
    let ppu = PPU::new(Rc::clone(&mem));
    let result_tile = ppu.get_tile(tile_id, false);
    assert_eq!(result_tile.data, test_sprite);
}

#[test]
fn test_ppu_get_bg_win_tile() {
    let mut mem = create_memory!();
    let test_sprite: [u8; TILE_SIZE] = [
        0x00, 0x3C,
        0x3C, 0x66,
        0x5A, 0xDB,
        0x5A, 0x81,
        0x7E, 0x99,
        0x5A, 0xDB,
        0x3C, 0x7E,
        0x00, 0x3C,
    ];
    let tile_id: u8 = 128;
    let tile_address = VRAM_BLOCK_1_ADDRESS;
    for i in 0..TILE_SIZE {
        mem.borrow_mut().write((tile_address + i) as u16, test_sprite[i]);
    }
    mem.borrow_mut().write(LCDC, 0xFF);
    let ppu = PPU::new(Rc::clone(&mem));
    let result_tile = ppu.get_tile(tile_id, true);
    assert_eq!(result_tile.data, test_sprite);

    mem.borrow_mut().write(LCDC, 0);
    for i in 0..TILE_SIZE {
        mem.borrow_mut().write((tile_address + i) as u16, 0);
    }
    let tile_id: u8 = 0;
    let tile_address = VRAM_BLOCK_2_ADDRESS;
    for i in 0..TILE_SIZE {
        mem.borrow_mut().write((tile_address + i) as u16, test_sprite[i]);
    }
    let result_tile = ppu.get_tile(tile_id, true);
    assert_eq!(result_tile.data, test_sprite);
}
