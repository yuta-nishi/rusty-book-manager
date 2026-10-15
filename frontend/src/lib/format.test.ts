import { describe, expect, it } from 'vite-plus/test';
import { formatDateTime, shortId } from '#lib/format.js';

describe('formatDateTime', () => {
	it('renders a placeholder for missing timestamps', () => {
		expect(formatDateTime(null)).toBe('—');
		expect(formatDateTime(undefined)).toBe('—');
		expect(formatDateTime('')).toBe('—');
	});

	it('passes unparseable values through instead of showing an invalid date', () => {
		expect(formatDateTime('not-a-date')).toBe('not-a-date');
	});

	it('formats an RFC 3339 timestamp', () => {
		const formatted = formatDateTime('2026-10-05T13:42:00Z');

		expect(formatted).not.toBe('—');
		expect(formatted).toContain('2026');
	});
});

describe('shortId', () => {
	it('abbreviates ids longer than eight characters', () => {
		expect(shortId('2bd5c35c7a8943cf9863cc65a6f23110')).toBe('2bd5c35c…');
	});

	it('keeps short ids intact', () => {
		expect(shortId('abc')).toBe('abc');
		expect(shortId('12345678')).toBe('12345678');
	});
});
