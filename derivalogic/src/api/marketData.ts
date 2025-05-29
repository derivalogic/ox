export async function fetchSpotRate(baseUrl: string, apiKey: string, symbol: string, date: string): Promise<number> {
  const url = `${baseUrl}/rest/v1/spot_rates?symbol=eq.${symbol}&date=eq.${date}&select=rate`;
  const resp = await fetch(url, {
    headers: {
      apikey: apiKey,
      Authorization: `Bearer ${apiKey}`,
    },
  });
  if (!resp.ok) {
    throw new Error(`failed to fetch spot rate: ${resp.status}`);
  }
  const data = await resp.json();
  return data?.[0]?.rate ?? 0;
}

export async function fetchCurve(baseUrl: string, apiKey: string, name: string): Promise<any> {
  const url = `${baseUrl}/rest/v1/curves?name=eq.${name}`;
  const resp = await fetch(url, {
    headers: {
      apikey: apiKey,
      Authorization: `Bearer ${apiKey}`,
    },
  });
  if (!resp.ok) {
    throw new Error(`failed to fetch curve: ${resp.status}`);
  }
  return resp.json();
}
