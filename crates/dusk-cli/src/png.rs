//! A small PNG writer for the tools' map pictures: RGB, 8 bits, uncompressed (stored deflate
//! blocks), so it needs no dependencies. The files are bigger than they need be; they're for
//! looking at.

use std::io::Write;

/// Writes `pixels` (RGB, row by row, `width × height × 3` bytes) as a PNG.
pub fn write(
    path: &std::path::Path,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> std::io::Result<()> {
    assert_eq!(pixels.len(), (width * height * 3) as usize);
    // Each row starts with its filter type: 0, none.
    let row = (width * 3) as usize;
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in pixels.chunks(row) {
        raw.push(0);
        raw.extend_from_slice(line);
    }
    // A zlib stream of stored blocks, at most 65,535 bytes each.
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(u8::from(i + 1 == blocks.len()));
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, RGB, deflate, no filter, no interlace

    let mut out = std::fs::File::create(path)?;
    out.write_all(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a])?;
    for (kind, data) in [
        (b"IHDR", &header[..]),
        (b"IDAT", &zlib[..]),
        (b"IEND", &[][..]),
    ] {
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        out.write_all(kind)?;
        out.write_all(data)?;
        let mut crc_input = kind.to_vec();
        crc_input.extend_from_slice(data);
        out.write_all(&crc32(&crc_input).to_be_bytes())?;
    }
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in bytes {
        a = (a + u32::from(x)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}
