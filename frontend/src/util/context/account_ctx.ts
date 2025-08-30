import { createContext } from 'react';
import type { AccountInfo } from '../loaders/auth';

export interface AccountContextData {
  account: AccountInfo;
  setAccount: React.Dispatch<React.SetStateAction<AccountInfo>>;
}

export const AccountContext = createContext<AccountContextData | null>(null);
