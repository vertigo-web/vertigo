export const getRandom = (min: number, max: number): number => {
    const range = max - min + 1;
    let result = Math.floor(Math.random() * range);
    return min + result;
};

