import {BB64Long, bitboardToString} from '../BB64Long';

export function normalizeBitboard(stringsOrString: TemplateStringsArray | string, ...values: any[]): string {
  let fullString: string;

  if (typeof stringsOrString === 'string') {
    // If input is a regular string, treat it directly
    fullString = stringsOrString;
  } else {
    // If input is a template literal, combine it with the values
    fullString = stringsOrString
      .map((str, i) => `${str}${values[i] || ''}`)
      .join('');
  }

  // Match the leading indentation from the first indented line
  const match = fullString.match(/^[ \t]*(?=\S)/gm);
  const indent = match ? Math.min(...match.map(el => el.length)) : 0;

  if (indent > 0) {
    return fullString.replace(new RegExp(`^[ \\t]{${indent}}`, 'gm'), '').trim();
  }

  return fullString.trim();
}

export function bitboardToNormalized(bb: BB64Long): string {
  return normalizeBitboard(bitboardToString(bb));
}
