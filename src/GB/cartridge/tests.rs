use crate::GB::cartridge::{Cartridge, CartridgeType};

#[test]
fn test_new() {
    let cart = Cartridge::new("resources/test/mbc1_rom_banks.gb".to_string()).unwrap();
    println!("{}", cart);
    assert_eq!(cart.get_rom_size(), 2 * 1024 * 1024);
    assert_eq!(cart.get_ram_size(), 8 * 1024);
    assert_eq!(cart.get_cart_type(), CartridgeType::Mbc1RamBattery);
}

#[test]
fn test_rom_bank() {
    todo!("Implement")
}

#[test]
fn test_ram_bank() {
    todo!("Implement")
}
