export function launchExternalUrl(
  rawUrl: string,
  navigate: (url: string) => void = (url) => window.location.assign(url),
) {
  const url = new URL(rawUrl);
  if (url.protocol !== 'https:') throw new Error('HTTPS以外のURLは開けません。');
  navigate(url.toString());
}
