// Unicode and emoji test file for edge case testing

// Chinese function name
function 计算总和(数字列表: number[]): number {
  return 数字列表.reduce((a, b) => a + b, 0);
}

// Variable with emoji and accents
const émoji = '🚀'; // Rocket emoji variable

// Japanese variable
const 日本語変数 = 'こんにちは';

// Arabic variable (right-to-left)
const متغير_عربي = 'مرحبا';

// Hebrew variable (right-to-left)
const משתנה_עברי = 'שלום';

// Mixed script identifier
const мixed_скрипт = 'Смешанный';

// Mathematical symbols (often multi-byte)
const π = 3.14159;
const ∑ = (arr: number[]) => arr.reduce((a, b) => a + b, 0);

// Complex emoji sequences
const 👨‍👩‍👧‍👦 = 'family'; // Family emoji with zero-width joiners

// Combining characters
const cõmbíñéd = 'combined with diacritics';

// Export for testing
export {
  计算总和,
  émoji,
  日本語変数,
  متغير_عربي,
  משתנה_עברי,
  мixed_скрипт,
  π,
  ∑,
  👨‍👩‍👧‍👦,
  cõmbíñéd
};