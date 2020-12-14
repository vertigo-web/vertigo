pub struct Config {
    pub border: u32,
    pub itemWidth: u32,
    pub itemPossibleWidth: u32,
    pub itemBorderSize: u32,
    pub itemWidthSize: u32,
    pub itemWidthSizeOuther: u32,
    pub groupBorderSize: u32,
    pub groupWidthSize: u32,
    pub groupWidthSizeOuther: u32,
    pub allWidth: u32,
}

impl Config {
    pub fn new() -> Config {
        let border = 1;
        let itemWidth = 80;
        let itemPossibleWidth = 20;
        let itemBorderSize = border;                                        //1
        let itemWidthSize = itemWidth;                                      //20
        let itemWidthSizeOuther = itemWidthSize + (2 * itemBorderSize);     //22
        let groupBorderSize = border;                                       //1
        let groupWidthSize = 3 * itemWidthSizeOuther;                       //66
        let groupWidthSizeOuther = groupWidthSize + (2 * groupBorderSize);  //68
        let allWidth = groupWidthSizeOuther * 3;                            //204

        Config {
            border,
            itemWidth,
            itemPossibleWidth,
            itemBorderSize,
            itemWidthSize,
            itemWidthSizeOuther,
            groupBorderSize,
            groupWidthSize,
            groupWidthSizeOuther,
            allWidth,
        }
    }
}
