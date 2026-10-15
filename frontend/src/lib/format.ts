/** Formats an RFC 3339 timestamp as a Japanese local date and time. */
export function formatDateTime(value: string | null | undefined): string {
	if (!value) return '—';
	const date = new Date(value);
	if (Number.isNaN(date.getTime())) return value;
	return date.toLocaleString('ja-JP', { dateStyle: 'medium', timeStyle: 'short' });
}

/** Shortens a UUID-ish book/checkout ID for display. */
export function shortId(id: string): string {
	return id.length > 8 ? `${id.slice(0, 8)}…` : id;
}
