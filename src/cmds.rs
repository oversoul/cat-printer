

fn to_unsigned_byte(val: i16) -> u8 {
	if val >= 0 {
		val.try_into().unwrap()
	} else {
		(val & 0xff).try_into().unwrap()
	}
}

fn bs(lst: &[i16]) -> Vec<u8> {
    lst.iter().map(|&x| to_unsigned_byte(x)).collect()
}

const CMD_GET_DEV_STATE : &[u8] = &[81, 120, 163, 0, 1, 0, 0, 0, 255];

// const CMD_SET_QUALITY_200_DPI  : &[u8] = &[81, 120, 164, 0, 1, 0, 50, 158, 255];

// const CMD_GET_DEV_INFO  : &[u8] = &[81, 120, 168, 0, 1, 0, 0, 0, 255];

const CMD_LATTICE_START : &[u8] = &[81, 120, 166, 0, 11, 0, 170, 85, 23, 56, 68, 95, 95, 95, 68, 56, 44, 161, 255];

const CMD_LATTICE_END : &[u8] = &[81, 120, 166, 0, 11, 0, 170, 85, 23, 0, 0, 0, 0, 0, 0, 0, 23, 17, 255];

const CMD_SET_PAPER : &[u8] = &[81, 120, 161, 0, 2, 0, 48, 0, 249, 255];

const CMD_PRINT_IMG : &[u8] = &[81, 120, 190, 0, 1, 0, 0, 0, 255];
const CMD_PRINT_TEXT : &[u8] = &[81, 120, 190, 0, 1, 0, 1, 7, 255];

const CHECKSUM_TABLE : &[u8] = &[0, 7, 14, 9, 28, 27, 18, 21, 56, 63, 54, 49, 36, 35, 42, 45, 112, 119, 126, 121, 108, 107, 98, 101, 72, 79, 70, 65, 84, 83, 90, 93, 224, 231, 238, 233, 252, 251, 242, 245, 216, 223, 214, 209, 196, 195, 202, 205, 144, 151, 158, 153, 140, 139, 130, 133, 168, 175, 166, 161, 180, 179, 186, 189, 199, 192, 201, 206, 219, 220, 213, 210, 255, 248, 241, 246, 227, 228, 237, 234, 183, 176, 185, 190, 171, 172, 165, 162, 143, 136, 129, 134, 147, 148, 157, 154, 39, 32, 41, 46, 59, 60, 53, 50, 31, 24, 17, 22, 3, 4, 13, 10, 87, 80, 89, 94, 75, 76, 69, 66, 111, 104, 97, 102, 115, 116, 125, 122, 137, 142, 135, 128, 149, 146, 155, 156, 177, 182, 191, 184, 173, 170, 163, 164, 249, 254, 247, 240, 229, 226, 235, 236, 193, 198, 207, 200, 221, 218, 211, 212, 105, 110, 103, 96, 117, 114, 123, 124, 81, 86, 95, 88, 77, 74, 67, 68, 25, 30, 23, 16, 5, 2, 11, 12, 33, 38, 47, 40, 61, 58, 51, 52, 78, 73, 64, 71, 82, 85, 92, 91, 118, 113, 120, 127, 106, 109, 100, 99, 62, 57, 48, 55, 34, 37, 44, 43, 6, 1, 8, 15, 26, 29, 20, 19, 174, 169, 160, 167, 178, 181, 188, 187, 150, 145, 152, 159, 138, 141, 132, 131, 222, 217, 208, 215, 194, 197, 204, 203, 230, 225, 232, 239, 250, 253, 244, 243];


#[allow(dead_code)]
fn chk_sum(b_arr: &[u8], i: usize, i2: usize) -> u8 {
    let mut b2 = 0;
    for i3 in i..(i + i2) {
        b2 = CHECKSUM_TABLE[usize::from((b2 ^ b_arr[i3]) & 0xff)];
    }
    b2
}

// def chk_sum(b_arr, i, i2):
//     b2 = 0
//     for i3 in range(i, i + i2):
//         b2 = CHECKSUM_TABLE[(b2 ^ b_arr[i3]) & 0xff]
//     return b2




#[allow(dead_code)]
fn cmd_feed_paper(how_much: u8) -> Vec<u8> {
    let b_arr = &mut [81, 120, 189, 0, 1, 0, how_much, 0, 0xff];
    b_arr[7] = chk_sum(b_arr, 6, 1);
    b_arr.to_vec()
}

#[allow(dead_code)]
fn cmd_set_energy(val: i16) -> Vec<u8> {
    let mut b_arr = bs(&[81, 120, -81, 0, 2, 0, (val >> 8) & 0xff, val, 0, 0xff]);
    b_arr[7] = chk_sum(&b_arr, 6, 2);
    b_arr.to_vec()
}

#[allow(dead_code)]
fn encode_run_length_repetition(n: u16, val: u16) -> Vec<u8> {
    let mut res: Vec<u8> = vec![];
    let mut v = n;
    while v > 0x7f {
        res.push((0x7f | (val << 7)).try_into().unwrap());
        v -= 0x7f;
    }

    if v > 0 {
        res.push(((val << 7) | v).try_into().unwrap());
    }
    res
}

#[allow(dead_code)]
fn run_length_encode(img_row: &Vec<u8>) -> Vec<u8> {
    let mut res = vec![];
    let mut count: u16 = 0;
    let mut last_val: u8 = 0;
    for &val in img_row {
        if val == last_val {
            count += 1;
        } else {
            res.extend(encode_run_length_repetition(count, last_val.into()));
            count = 1;
        }
        last_val = val;
    }

    if count > 0 {
        if count > 255 { count = 255; }
        res.extend(encode_run_length_repetition(count, last_val.into()));
    }
    res
}

fn bit_encode(img_row: &Vec<u8>, chunk_start: usize, bit_index: usize) -> u8 {
    if bit_index >= img_row.len() {
        return 255;
    }
    if img_row[chunk_start + bit_index] > 0 {
         1 << bit_index
    } else {
        0
    }
}

#[allow(dead_code)]
fn byte_encode(img_row: Vec<u8>) -> Vec<u8> {
    let mut res = vec![];
    for chunk_start in (0..img_row.len()).step_by(8) {
        let mut byte = 0;
        for bit_index in 0..8 {
            byte |= bit_encode(&img_row, chunk_start, bit_index);
        }
        res.push(byte);
    }
    res
}

#[allow(dead_code)]
fn cmd_print_row(img_row: &Vec<u8>) -> Vec<u8> {
    // Try to use run-length compression on the image data.
    let encoded_img = run_length_encode(img_row);

    // println!("{:?}", encoded_img.len());

    // let encoded_img = &img_row;

    let encoded_img_len = encoded_img.len();
    // Build the run-length encoded image command.
    let row : Vec<u8> = vec![81, 120, 191, 0, encoded_img.len().try_into().unwrap(), 0];
    let mut b_arr = [row, encoded_img.to_vec(), [0, 0xff].to_vec()].concat();
    let b_arr_len = b_arr.len();

    b_arr[b_arr_len - 2] = chk_sum(&b_arr, 6, encoded_img_len);

    b_arr
}

#[allow(dead_code)]
pub enum PrinterMode {
    Text,
    Image,
}

fn quality_cmd() -> Vec<u8> {
    [81, 120, 161, 0, 2, 0, 72, 0, 243, 255].to_vec()
}

pub fn cmds_print_img(img: Vec<Vec<u8>>, mode: PrinterMode) -> Vec<u8> {

    let printer_mode = match mode {
        PrinterMode::Text => CMD_PRINT_TEXT,
        PrinterMode::Image => CMD_PRINT_IMG,
    };

    let mut data = vec![];

    data.append(&mut CMD_GET_DEV_STATE.to_vec());
    data.append(&mut quality_cmd());
    data.append(&mut printer_mode.to_vec());
    data.append(&mut CMD_LATTICE_START.to_vec());

    for row in img {
        data.append(&mut cmd_print_row(&row));
    }

    data.append(&mut cmd_feed_paper(25));
    data.append(&mut CMD_SET_PAPER.to_vec());
    data.append(&mut CMD_SET_PAPER.to_vec());
    data.append(&mut CMD_SET_PAPER.to_vec());
    data.append(&mut CMD_LATTICE_END.to_vec());
    data.append(&mut CMD_GET_DEV_STATE.to_vec());

    data
}

#[cfg(test)]
mod tests {

    use crate::cmds::chk_sum;

    #[test]
    fn calculate_checksum() {
        let encoded_img = vec![1, 129];
        let encoded_img_len = 2;
        let row : Vec<u8> = vec![81, 120, 191, 0, 2, 0];
        let b_arr = [row, encoded_img, [0, 0xff].to_vec()].concat();
        assert_eq!(155, chk_sum(&b_arr, 6, encoded_img_len));
    }

    #[test]
    fn calculate_checksum_1() {
        let encoded_img = vec![2];
        let encoded_img_len = encoded_img.len();
        let row : Vec<u8> = vec![81, 120, 191, 0, 2, 0];
        let b_arr = [row, encoded_img, [0, 0xff].to_vec()].concat();
        assert_eq!(14, chk_sum(&b_arr, 6, encoded_img_len));
    }


}
