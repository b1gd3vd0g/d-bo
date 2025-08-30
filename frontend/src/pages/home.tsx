import { useNavigate } from 'react-router-dom';
import {
  AccountContext,
  type AccountContextData
} from '../util/context/account_ctx';
import { useContext } from 'react';

export default function HomePage() {
  const navigate = useNavigate();
  const { account } = useContext(AccountContext) as AccountContextData;

  return (
    <>
      <h1>Welcome to D-Bo, {account.username}</h1>
      <button
        onClick={() => {
          sessionStorage.removeItem('token');
          navigate('/welcome');
        }}
      >
        Log out
      </button>
    </>
  );
}
