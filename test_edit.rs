fn main() {
    let mut self_str = "😄".to_string(); // 4 bytes
    let start_byte = 1;
    let old_end_byte = 3;
    let inserted_text = b"x".to_vec();

    let safe_start = {
        let mut s = start_byte;
        while s > 0 && !self_str.is_char_boundary(s) {
            s -= 1;
        }
        s
    };
    let safe_end = {
        let mut e = old_end_byte;
        while e <= self_str.len() && !self_str.is_char_boundary(e) {
            e += 1;
        }
        e
    };

    println!("safe_start: {}, safe_end: {}", safe_start, safe_end);

    // This panics if start_byte is not a char boundary!
    // self_str[safe_start..start_byte]
    // Ah!! Because self_str is a string, and start_byte is not a char boundary!
    // So we should do self_str.as_bytes()[safe_start..start_byte]
}
