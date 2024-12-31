
// http://www.mpgedit.org/mpgedit/mpeg_format/mpeghdr.htm
struct Last128Bytes {
  tag: [char; 3], // bytes 0-2; should be TAG for ID3v1
  title: [char; 30], // bytes 3-32
  artist: [char; 30], // bytes 33-62
  album: [char; 30], // bytes 63-92
  year: [char; 4], // bytes 93-96
  comment: [char; 30], // bytes 97-126
  genre: u8 // byte 127
}