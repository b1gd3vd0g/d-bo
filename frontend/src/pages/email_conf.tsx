export function ConfirmEmailPage() {
  return (
    <>
      <h4>Confirm your email address to start playing D-Bo!</h4>
      <button className='block mx-auto'>Confirm email</button>
    </>
  );
}

export function RejectEmailPage() {
  return (
    <>
      <h4>Remove your email address from our records:</h4>
      <p>
        A player has registered your email address for a new D-Bo account! If
        this is a mistake, use the button below to remove all information from
        our databases.
      </p>
      <button className='block mx-auto'>Remove account</button>
    </>
  );
}
