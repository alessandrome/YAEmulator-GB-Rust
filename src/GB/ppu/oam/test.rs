#[cfg(test)]
mod test {
    use super::super::{OAM, AttributesMasks};

    macro_rules! new_oam {
        ($oam: ident, $x_ident: ident, $y_ident: ident, $tile_ident: ident, $id_ident: ident, $attr_ident: ident, $x: expr, $y: expr, $tile_id: expr, $id: expr, $attributes: expr) => {
            let ($y_ident, $x_ident, $tile_ident, $id_ident) = ($y, $x, $tile_id, $id);
            let $attr_ident: u8 = $attributes;
            let $oam = OAM::new($y_ident, $x_ident, $tile_ident, $attr_ident, $id_ident);
        };
    }

    #[test]
    fn new_oam() {
        new_oam!(oam, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(1), 0b1111_0000);
        assert_eq!(oam.y, test_y);
        assert_eq!(oam.x, test_x);
        assert_eq!(oam.tile_id, test_tile_id);
        assert_eq!(oam.id, test_id);
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, true);
        assert_eq!(oam.x_flip, true);
        assert_eq!(oam.palette, true);
    }

    #[test]
    fn get_oam_bytes() {
        new_oam!(oam, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, None, 0b1001_0110);
        assert_eq!(oam.id, None);
        assert_eq!(oam.priority, true);
        assert_eq!(oam.y_flip, false);
        assert_eq!(oam.x_flip, false);
        assert_eq!(oam.palette, true);
        let oam_bytes = oam.get_oam_bytes();
        assert_eq!(oam_bytes.y, test_y);
        assert_eq!(oam_bytes.x, test_x);
        assert_eq!(oam_bytes.tile_id, test_tile_id);
        assert_eq!(oam_bytes.attributes, attributes);
    }

    #[test]
    fn test_order() {
        new_oam!(oam_1, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(1), 0b1001_0110);
        new_oam!(oam_2, test_x, test_y, test_tile_id, test_id, attributes, 56u8, 131u8, 33u8, Some(2), 0b1001_0110);
        new_oam!(oam_3, test_x, test_y, test_tile_id, test_id, attributes, 39u8, 131u8, 33u8, Some(3), 0b1001_0110);
        assert_eq!(oam_1 < oam_2, true);
        assert_eq!(oam_2 < oam_3, false);
    }
}
