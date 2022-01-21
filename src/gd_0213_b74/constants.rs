#[rustfmt::skip]
// Original Waveforms from Waveshare
pub(crate) const LUT_FULL_UPDATE: [u8; 70] =[
    0x80,0x60,0x40,0x00,0x00,0x00,0x00,             // LUT0: BB:     VS 0 ~7
    0x10,0x60,0x20,0x00,0x00,0x00,0x00,             // LUT1: BW:     VS 0 ~7
    0x80,0x60,0x40,0x00,0x00,0x00,0x00,             // LUT2: WB:     VS 0 ~7
    0x10,0x60,0x20,0x00,0x00,0x00,0x00,             // LUT3: WW:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT4: VCOM:   VS 0 ~7

    0x03,0x03,0x00,0x00,0x02,                       //  TP0 A~D RP0
    0x09,0x09,0x00,0x00,0x02,                       //  TP1 A~D RP1
    0x03,0x03,0x00,0x00,0x02,                       //  TP2 A~D RP2
    0x00,0x00,0x00,0x00,0x00,                       //  TP3 A~D RP3
    0x00,0x00,0x00,0x00,0x00,                       //  TP4 A~D RP4
    0x00,0x00,0x00,0x00,0x00,                       //  TP5 A~D RP5
    0x00,0x00,0x00,0x00,0x00,                       //  TP6 A~D RP6
];

#[rustfmt::skip]
pub(crate) const LUT_PARTIAL_UPDATE: [u8; 70] =[
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT0: BB:     VS 0 ~7
    0x80,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT1: BW:     VS 0 ~7
    0x40,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT2: WB:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT3: WW:     VS 0 ~7
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,             // LUT4: VCOM:   VS 0 ~7

    0x0A,0x00,0x00,0x00,0x00,                       //  TP0 A~D RP0
    0x00,0x00,0x00,0x00,0x00,                       //  TP1 A~D RP1
    0x00,0x00,0x00,0x00,0x00,                       //  TP2 A~D RP2
    0x00,0x00,0x00,0x00,0x00,                       //  TP3 A~D RP3
    0x00,0x00,0x00,0x00,0x00,                       //  TP4 A~D RP4
    0x00,0x00,0x00,0x00,0x00,                       //  TP5 A~D RP5
    0x00,0x00,0x00,0x00,0x00,                       //  TP6 A~D RP6
];
