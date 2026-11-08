import axios from 'axios';
export const api = axios.create({ baseURL: '/api', timeout: 15000 });
export function errorMessage(error: unknown): string {
  if (axios.isAxiosError(error)) {
    return typeof error.response?.data?.error === 'string' ? error.response.data.error : error.response ? 'Something went wrong. Please try again.' : 'Cannot reach the server. Please try again.';
  }
  return 'Something went wrong. Please try again.';
}
