interface ContainerProps {
  children: React.ReactElement;
  /** The extra class names to add. These will override any conflicting default classes. */
  className?: string;
}

/**
 * Create a box around child elements with default formatting.
 */
export function Box({ children, className = '' }: ContainerProps) {
  return (
    <div className={`w-fit rounded-md bg-gray-400 p-2 ${className}`}>
      {children}
    </div>
  );
}
