pub struct Config {
    pub border: u32,
    pub item_width: u32,
    pub item_possible_width: u32,
    pub item_border_size: u32,
    pub item_width_size: u32,
    pub item_width_size_outher: u32,
    pub group_border_size: u32,
    pub group_width_size: u32,
    pub group_width_size_outher: u32,
    pub all_width: u32,
}

impl Config {
    pub fn new() -> Config {
        let border = 1;
        let item_width = 70;
        let item_possible_width = 20;
        let item_border_size = border; // 1
        let item_width_size = item_width; // 20
        let item_width_size_outher = item_width_size + (2 * item_border_size); // 22
        let group_border_size = border; // 1
        let group_width_size = 3 * item_width_size_outher; // 66
        let group_width_size_outher = group_width_size + (2 * group_border_size); // 68
        let all_width = group_width_size_outher * 3; // 204

        Config {
            border,
            item_width,
            item_possible_width,
            item_border_size,
            item_width_size,
            item_width_size_outher,
            group_border_size,
            group_width_size,
            group_width_size_outher,
            all_width,
        }
    }
}
