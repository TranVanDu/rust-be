pub fn format_number(number: i64) -> String {
  let mut s = number.to_string();
  let length = s.len();
  let mut pos: usize = length;

  // Xử lý số âm (bỏ qua dấu '-' nếu có)
  if number < 0 {
    pos -= 1; // Tránh đặt dấu chấm ngay sau dấu âm
  }

  // Chèn dấu chấm từ phải sang trái mỗi 3 chữ số
  while pos > 3 {
    pos -= 3;
    s.insert(pos, '.');
  }
  s
}
