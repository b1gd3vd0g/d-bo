import { useNavigate } from 'react-router-dom';

export default function WelcomePage() {
  const navigate = useNavigate();
  return (
    <>
      <h1>Welcome to D-Bo!</h1>
      <button
        onClick={() => {
          sessionStorage.setItem('token', '123');
          navigate('/');
        }}
      >
        Log in
      </button>
    </>
  );
}
