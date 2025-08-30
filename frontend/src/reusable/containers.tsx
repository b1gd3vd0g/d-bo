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
    <div className={`w-fit p-2 bg-gray-400 rounded-md ${className}`}>
      {children}
    </div>
  );
}
