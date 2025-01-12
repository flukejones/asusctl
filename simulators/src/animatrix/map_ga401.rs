use super::Row;

pub const GA401: [Row; 63] = [
    Row(0x01, 7, 32, 0),
    Row(0x01, 7 + 34, 32, 0),
    Row(0x01, 7 + 68, 32, 0),
    Row(0x01, 7 + 102, 32, 0), // 34 len
    Row(0x01, 7 + 136, 32, 0),
    Row(0x01, 7 + 170, 34, 0),
    Row(0x01, 7 + 204, 34, 0),
    Row(0x01, 7 + 238, 34, 0),
    Row(0x01, 7 + 272, 34, 0),
    Row(0x01, 7 + 306, 34, 0),
    Row(0x01, 7 + 340, 34, 0),
    Row(0x01, 7 + 374, 34, 0),
    Row(0x01, 7 + 408, 33, 1),
    Row(0x01, 7 + 441, 33, 1),
    Row(0x01, 7 + 474, 32, 2),
    Row(0x01, 7 + 506, 32, 2),
    Row(0x01, 7 + 538, 31, 3),
    Row(0x01, 7 + 569, 31, 3),
    Row(0x01, 7 + 600, 28, 4),
    //
    Row(0x74, 7 + 1, 3, 28 + 4), // adds to end of previous
    Row(0x74, 7 + 3, 30, 4),
    Row(0x74, 7 + 33, 29, 5),
    Row(0x74, 7 + 62, 29, 5),
    Row(0x74, 7 + 91, 28, 6),
    Row(0x74, 7 + 119, 28, 6),
    Row(0x74, 7 + 147, 27, 7),
    Row(0x74, 7 + 174, 27, 7),
    Row(0x74, 7 + 202, 26, 9),
    Row(0x74, 7 + 228, 26, 9),
    Row(0x74, 7 + 254, 25, 10),
    Row(0x74, 7 + 278, 25, 9), // WEIRD OFFSET
    Row(0x74, 7 + 303, 24, 10),
    Row(0x74, 7 + 327, 24, 10),
    Row(0x74, 7 + 351, 23, 11),
    Row(0x74, 7 + 374, 23, 11),
    Row(0x74, 7 + 397, 22, 12),
    Row(0x74, 7 + 419, 22, 12),
    Row(0x74, 7 + 441, 21, 13),
    Row(0x74, 7 + 462, 21, 13),
    Row(0x74, 7 + 483, 20, 14),
    Row(0x74, 7 + 503, 20, 14),
    Row(0x74, 7 + 523, 19, 15),
    Row(0x74, 7 + 542, 19, 15),
    Row(0x74, 7 + 561, 18, 16),
    Row(0x74, 7 + 579, 18, 16),
    Row(0x74, 7 + 597, 17, 17),
    Row(0x74, 7 + 614, 13, 17),
    //
    Row(0xe7, 7 + 1, 4, 13 + 18), // adds to end of previous
    Row(0xe7, 7 + 4, 16, 18),
    Row(0xe7, 7 + 20, 16, 18),
    Row(0xe7, 7 + 36, 15, 19),
    Row(0xe7, 7 + 51, 15, 19),
    Row(0xe7, 7 + 66, 14, 20),
    Row(0xe7, 7 + 80, 12, 20), // too long? 14
    Row(0xe7, 7 + 94, 13, 21),
    Row(0xe7, 7 + 107, 13, 21),
    Row(0xe7, 7 + 120, 12, 12), // Actual display end
    Row(0xe7, 7 + 132, 12, 22),
    Row(0xe7, 7 + 144, 11, 23),
    Row(0xe7, 7 + 155, 11, 23),
    Row(0xe7, 7 + 166, 10, 24),
    Row(0xe7, 7 + 176, 10, 24),
    Row(0xe7, 7 + 186, 9, 25)
];
