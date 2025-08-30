import { redirect } from 'react-router-dom';

/**
 * This holds information about a single player's account.
 */
export interface AccountInfo {
  username: string;
  user_id: string;
  display_name: string;
}

/**
 * Make HTTP requests to find a player's account info based on the token stored
 * in session storage.
 */
export async function authLoader(): Promise<AccountInfo> {
  const token = sessionStorage.getItem('token');
  if (!token) throw redirect('/welcome');

  return {
    username: 'test_user',
    user_id: '123',
    display_name: 'test user'
  };
}
