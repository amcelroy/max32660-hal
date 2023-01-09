use max32660_pac;

const MAX32660_MAX_CLK: u32 = 96_000_000;

pub enum ClkPreScaler {
    DIV_1,
    DIV_2,
    DIV_4,
    DIV_8,
    DIV_16,
    DIV_32,
    DIV_64,
    DIV_128,
}

fn compute_clk_freq(max_clk_freq: u32, prescaler: u8) -> u32 {
    max_clk_freq / u32::pow(2, prescaler as u32)
}

pub fn configure_clk(gcr: &max32660_pac::GCR, psc: ClkPreScaler) -> u32 {
    let psc_u8 = psc as u8;
    

    gcr.clkcn.write(|w| {
        w.psc().bits(psc_u8)
    });

    let freq = compute_clk_freq(MAX32660_MAX_CLK, psc_u8);
    freq
}

pub fn get_clk_freq(gcr: &max32660_pac::GCR) -> u32 {
    let psc = gcr.clkcn.read().psc().bits();
    compute_clk_freq(MAX32660_MAX_CLK, psc)
}

/// Gets the peripheral clock
pub fn get_pclk_freq(gcr: &max32660_pac::GCR) -> u32 {
    get_clk_freq(gcr) / 2
}

