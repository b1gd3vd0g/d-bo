import { useState } from 'react';
import { FormGroup } from '../reusable/form';
import { Box } from '../reusable/containers';

type UserChoice = 'undecided' | 'login' | 'register';

interface WelcomeFormProps {
  setChoice: React.Dispatch<React.SetStateAction<UserChoice>>;
}

export default function WelcomePage() {
  const [choice, setChoice] = useState<UserChoice>('undecided');
  let form;
  switch (choice) {
    case 'undecided':
      form = <ChoiceForm setChoice={setChoice} />;
      break;
    case 'login':
      form = <LoginForm setChoice={setChoice} />;
      break;
    case 'register':
      form = <RegisterForm setChoice={setChoice} />;
      break;
  }

  return (
    <>
      <h1>Welcome to D-Bo!</h1>
      <Box className='m-auto'>{form}</Box>
    </>
  );
}

function ChoiceForm({ setChoice }: WelcomeFormProps) {
  return (
    <>
      <button className='w-1/1' onClick={() => setChoice('login')}>
        Log in
      </button>
      <button className='bg-accent-2' onClick={() => setChoice('register')}>
        Create a new account
      </button>
    </>
  );
}

function LoginForm({ setChoice }: WelcomeFormProps) {
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');

  console.log(username, password);

  return (
    <>
      <button
        className='bg-transparent text-3xl'
        onClick={() => setChoice('undecided')}
      >
        {'<'}
      </button>
      <h4 className='text-center'>Login</h4>
      <FormGroup label='Username/Email' setter={setUsername} />
      <FormGroup label='Password' setter={setPassword} />
    </>
  );
}

function RegisterForm({ setChoice }: WelcomeFormProps) {
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');
  const [email, setEmail] = useState<string>('');

  console.log(username, password, email);

  return (
    <>
      <button
        className='bg-transparent text-3xl'
        onClick={() => setChoice('undecided')}
      >
        {'<'}
      </button>
      <h4 className='text-center'>Create a new account</h4>
      <FormGroup label='Username' setter={setUsername} />
      <FormGroup label='Password' setter={setPassword} />
      <FormGroup label='Email address' setter={setEmail} />
    </>
  );
}
