type FormGroupProps = {
  /** The label for the input field. */
  label: string;
  /** The placeholder value in the input field. */
  hint?: string;
  /**
   * The number of rows in the field. 1 (default) creates an `input` field;
   * greater than 1 creates a `textarea`.
   */
  rows?: number;
  /**
   * The type of the input field. Only important if `rows === 1`. Critical for
   * password fields.
   */
  type?: string;
  /** A function to set the state variable. */
  setter: React.Dispatch<React.SetStateAction<string>>;
  /** A function to automatically format the input within the field as you type. */
  formatter?: (s: string) => string;
  /** A function to ensure valid input when the field leaves focus. */
  validator?: (s: string) => boolean;
};

export function TextFormGroup({
  label,
  hint = '',
  rows = 1,
  type = 'text',
  setter,
  formatter = (s) => s,
  validator = (s) => (s ? true : false)
}: FormGroupProps) {
  const id = label.toLowerCase().replace(/\W/g, '-');

  const formatInput = (
    event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    event.target.value = formatter(event.target.value);
  };

  const validateInput = (
    event: React.FocusEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const { value } = event.target;
    const valid = validator(value);
    if (valid || !value) {
      // Either the field is empty, or the value inside is valid.
      setter(value);
    } else if (value) {
      // The value is invalid! Highlight the field to reflect that,
      // and set the state variable to the empty string.
      setter('');
    }
  };

  const textarea = (
    <textarea
      id={id}
      name={id}
      placeholder={hint}
      onBlur={validateInput}
      onChange={formatInput}
      rows={rows}
    ></textarea>
  );

  const input = (
    <input
      type={type}
      id={id}
      name={id}
      placeholder={hint}
      onBlur={validateInput}
      onChange={formatInput}
    />
  );

  const field = rows > 1 ? textarea : input;

  // We always want text areas to be on their own line, below the label.
  const flex = rows > 1 ? 'flex-col' : 'flex';

  return (
    <div className={`${flex} m-2 flex-wrap justify-between`}>
      <label htmlFor={id} className='text-fg-1 text-xl'>
        {label}
      </label>
      {field}
    </div>
  );
}
