pub struct PacketUtils {

}
impl PacketUtils {
    pub fn write_packet(mut packet: Vec<u8>, id: u8, session_id: i32) -> Vec<u8> {
        let mut vec = vec![];
        vec.push(0xFE);
        vec.push(0xFD);
        vec.push(id);
        vec.append(&mut session_id.to_be_bytes().to_vec());
        vec.append(&mut packet);
        return vec;
    }
    pub fn write_string(string: String) -> Vec<u8> {
        let mut vec = vec![];
        vec.append(&mut string.as_bytes().to_vec());
        vec.push(0x00);
        return vec;
    }
    pub fn read_string(reader: &mut dyn std::io::Read) -> Option<String> {
        let mut string = vec![];
        loop {
            let mut byte = [0; 1];
            reader.read_exact(&mut byte).ok()?;
            if byte[0] != 0x00 {
                string.push(byte[0]);
            } else {
                break;
            }
        }
        Some(String::from_utf8_lossy(&string).to_string())
    }
    pub fn read_byte(reader: &mut dyn std::io::Read) -> Option<u8> {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte).ok()?;
        Some(byte[0])
    }
    pub fn read_int32(reader: &mut dyn std::io::Read) -> Option<i32> {
        let mut byte = [0; 4];
        reader.read_exact(&mut byte).ok()?;
        let int32 = i32::from_be_bytes(byte);
        Some(int32)
    }
}
#[derive(Clone)]
pub enum Element {
    StringElement { val: String },
    Byte { val: u8 },
    Short { val: i16 },
    Int32 { val: i32 },
    Bytes { val: Vec<u8> }
}
pub struct PacketBuilder {
    elements: Vec<Element>,
}
impl PacketBuilder {
    pub fn new() -> Self {
        return Self {
            elements: Vec::new(),
        };
    }
    pub fn insert_string(&mut self, string: &str) {
        self.elements.push(Element::StringElement {
            val: string.to_string(),
        });
    }
    pub fn insert_bytearray(&mut self, bytes: Vec<u8>) {
        self.elements
            .push(Element::Bytes { val: bytes });
    }
    pub fn insert_byte(&mut self, byte: u8) {
        self.elements.push(Element::Byte { val: byte });
    }
    pub fn insert_short(&mut self, short: i16) {
        self.elements.push(Element::Short { val: short });
    }
    pub fn insert_int(&mut self, int: i32) {
        self.elements.push(Element::Int32 { val: int });
    }
    pub fn build(mut self, id: u8, session_id: i32) -> Vec<u8> {
        let packet = self.internal_builder();
        let packet = PacketUtils::write_packet(packet, id, session_id);
        return packet;
    }
    pub fn internal_builder(&mut self) -> Vec<u8> {
        let mut packet = vec![];
        for element in self.elements.clone() {
            match element.clone() {
                Element::StringElement { val } => {
                    packet.append(&mut PacketUtils::write_string(val.clone()));
                }
                Element::Byte { val } => {
                    packet.push(val.to_le_bytes()[0]);
                }
                Element::Short { val } => {
                    packet.append(&mut val.to_be_bytes().to_vec());
                }
                Element::Int32 { val } => {
                    packet.append(&mut val.to_be_bytes().to_vec());
                }
                Element::Bytes { mut val } => {
                    packet.append(&mut val);
                }
            }
        }
        return packet;
    }
}
