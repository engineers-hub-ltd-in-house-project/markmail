import { describe, expect, it } from 'vitest';

describe('基本テスト', () => {
  it('1 + 1 は 2 になる', () => {
    expect(1 + 1).toBe(2);
  });

  it('文字列の結合', () => {
    expect('Hello' + ' ' + 'World').toBe('Hello World');
  });
});

// 基本的なユーティリティ関数のテスト
describe('ユーティリティ関数', () => {
  const isValidEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  it('有効なメールアドレスを検証', () => {
    expect(isValidEmail('test@example.com')).toBe(true);
    expect(isValidEmail('user@domain.org')).toBe(true);
  });

  it('無効なメールアドレスを検証', () => {
    expect(isValidEmail('invalid-email')).toBe(false);
    expect(isValidEmail('test@')).toBe(false);
    expect(isValidEmail('@example.com')).toBe(false);
    expect(isValidEmail('')).toBe(false);
  });
});
