import axios, { type AxiosResponse } from "axios";

async function handleRequest(
  req: () => Promise<AxiosResponse>
): Promise<{ message: string; data: any }> {
  try {
    const res = await req();
    return { message: res.data.message, data: res.data.data };
  } catch (err: any) {
    let msg = "Request failed";
    if (err?.response) {
      const res = err.response as AxiosResponse;
      if (typeof res.data === "string") {
        if (err.response.data) {
          msg = err.response.data;
        } else {
          msg = `${res.statusText}: ${res.status}`;
        }
      } else {
        msg = err.response.data.message;
      }
    } else {
      msg = `Request failed: ${err}`;
    }
    throw msg;
  }
}

export async function requestGet(
  url: string,
  params?: Record<string, string | number>
): Promise<{ message: string; data: any }> {
  return await handleRequest(() => axios.get(url, { params }));
}

export async function requestPost(
  url: string,
  data?: Record<string, any>,
  params?: Record<string, string | number>
): Promise<{ message: string; data: any }> {
  return await handleRequest(() => axios.post(url, data, { params }));
}
