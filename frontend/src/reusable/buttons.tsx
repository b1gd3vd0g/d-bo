interface ButtonProps {
  children: React.ReactElement | string;
  /** The extra class names to add. These will override any conflicting default classes. */
  className?: string;
  /** The function to be executed when the button is clicked. */
  onClick: () => void;
}

export function PrimaryButton({
  children,
  className = '',
  onClick
}: ButtonProps) {
  return (
    <button
      className={`bg-accent-1 text-fg-2 mx-auto my-4 block rounded-xl px-4 py-2 text-2xl ${className}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
}

export function SecondaryButton({
  children,
  className = '',
  onClick
}: ButtonProps) {
  return (
    <button
      className={`bg-accent-2 text-fg-2 mx-auto my-4 block rounded-xl px-4 py-2 text-2xl ${className}`}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
