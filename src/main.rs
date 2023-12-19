use std::net::UdpSocket;

// The whole must be 12 bytes when encoded.
// which is 3 x 4 bytes.
#[derive(Default)]
struct Header {
    packet_id: u16, // 16 bit

    query_response_indicator: bool, // 1  bit
    operation_code: u8,             // 4  bits
    authoritative_answer: bool,     // 1  bit
    truncation: bool,               // 1  bit
    recursion_desired: bool,        // 1  bit

    recursion_available: bool, // 1  bit
    reserved: u8,              // 3  bits
    response_code: u8,         // 4  bits

    question_count: u16,          // 16 bits
    answer_record: u16,           // 16 bits
    authority_record_count: u16,  // 16 bits
    additional_record_count: u16, // 16 bits
}

impl Header {
    fn new(id: u16) -> Self {
        Self {
            packet_id: id.to_be(),
            query_response_indicator: true,
            ..Self::default()
        }
    }

    fn encode(&self) -> [u8; 12] {
        let part_two: u8 = (self.query_response_indicator as u8) << 7
            | (self.operation_code << 3)
            | (self.authoritative_answer as u8) << 2
            | (self.truncation as u8) << 1
            | (self.recursion_desired as u8);

        let part_three: u8 =
            (self.recursion_available as u8) << 7 | (self.reserved as u8) << 4 | self.response_code;

        [
            (self.packet_id >> 8) as u8,
            self.packet_id as u8,
            part_two,
            part_three,
            (self.question_count >> 8) as u8,
            self.question_count as u8,
            (self.answer_record >> 8) as u8,
            self.answer_record as u8,
            (self.authority_record_count >> 8) as u8,
            self.authority_record_count as u8,
            (self.additional_record_count >> 8) as u8,
            self.additional_record_count as u8,
        ]
    }
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];
    let header = Header::new(1234);

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let response = header.encode();
                udp_socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
