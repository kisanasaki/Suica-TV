const ALLOWED_EXTERNAL_HOSTS = new Set([
  'www.youtube.com',
  'www.google.com',
  'zaim.net',
]);

export function launchExternalUrl(
  rawUrl: string,
  navigate: (url: string) => void = (url) => window.location.assign(url),
) {
  const url = new URL(rawUrl);
  if (url.protocol !== 'https:') throw new Error('HTTPS以外のURLは開けません。');
  if (!ALLOWED_EXTERNAL_HOSTS.has(url.hostname)) {
    throw new Error('許可されていないURLは開けません。');
  }
  navigate(url.toString());
}
