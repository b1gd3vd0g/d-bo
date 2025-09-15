import { useState } from 'react';
import { FormGroup } from '../reusable/form';
import { Box } from '../reusable/containers';
import {
  BackButton,
  PrimaryButton,
  SecondaryButton
} from '../reusable/buttons';

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
      <h1 className='pt-12'>Welcome to D-Bo!</h1>
      <p className='mx-auto my-4 w-4/5 text-center'>
        D-Bo is an online space where players can connect with each other and
        play a classic card game together in real time.
      </p>
      <Box className='m-auto'>{form}</Box>
    </>
  );
}

function ChoiceForm({ setChoice }: WelcomeFormProps) {
  return (
    <>
      <PrimaryButton className='w-1/1' onClick={() => setChoice('login')}>
        Log in
      </PrimaryButton>
      <SecondaryButton onClick={() => setChoice('register')}>
        Create a new account
      </SecondaryButton>
    </>
  );
}

function LoginForm({ setChoice }: WelcomeFormProps) {
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');

  console.log(username, password);

  return (
    <>
      <BackButton onClick={() => setChoice('undecided')} />
      <h2 className='text-center'>Log in:</h2>
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
      <BackButton onClick={() => setChoice('undecided')} />{' '}
      <button
        className='bg-transparent text-3xl'
        onClick={() => setChoice('undecided')}
      >
        {'<'}
      </button>
      <h2 className='text-center'>Create a new account:</h2>
      <FormGroup label='Username' setter={setUsername} />
      <FormGroup label='Password' setter={setPassword} />
      <FormGroup label='Email address' setter={setEmail} />
    </>
  );
}
