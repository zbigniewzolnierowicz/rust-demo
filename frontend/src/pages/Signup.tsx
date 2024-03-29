import clsx from "clsx";
import { FC, forwardRef } from "react";
import { Field, Form } from "react-final-form";
import { useLoginState } from "../stores/login";
import { Navigate } from "react-router-dom";
import { validate } from "email-validator"
import { CreateNewUserDTO } from "common/bindings/CreateNewUserDTO";

const Input = forwardRef<HTMLInputElement, JSX.IntrinsicElements["input"]>(
  ({ className, ...props }, ref) => (
    <input
      {...props}
      className={clsx("bg-slate-200 border border-black p-2", className)}
      ref={ref}
    />
  ),
);

export const Signup: FC = () => {
  const { userData, createAccount } = useLoginState();

  if (userData !== null) {
    return <Navigate to="/" />;
  }

  return (
    <div className="max-w-screen-md mx-auto p-4">
      <Form
        onSubmit={(v: CreateNewUserDTO) => createAccount(v)}
        validate={(v) => {
          const errors: Record<string, string> = {};

          if (!v.username) {
            errors.username = "Username is required.";
          }

          if (!v.password) {
            errors.password = "Password is required.";
          }

          if (!v.email) {
            errors.email = "Email is required.";
          } else {
            if (!validate(v.email)) {
              errors.email = "Email must be valid";
            }
          }

          return errors;
        }}
      >
        {(props) => (
          <form
            onSubmit={props.handleSubmit}
            className="flex flex-col bg-zinc-300 text-black"
          >
            <Field name="username" id="signup-username">
              {({ input, meta }) => (
                <>
                  <label htmlFor={input.name}>
                    Username
                    {meta.error && (
                      <span className="text-red-600">{meta.error}</span>
                    )}
                  </label>
                  <Input {...input} />
                </>
              )}
            </Field>
            <Field name="password" id="signup-password">
              {({ input, meta }) => (
                <>
                  <label htmlFor={input.name}>
                    Password
                    {meta.error && (
                      <span className="text-red-600">{meta.error}</span>
                    )}
                  </label>
                  <Input
                    autoComplete="new-password"
                    type="password"
                    {...input}
                  />
                </>
              )}
            </Field>
            <Field name="email" id="signup-email" type="email">
              {({ input, meta }) => (
                <>
                  <label htmlFor={input.name}>
                    Email
                    {meta.error && (
                      <span className="text-red-600">{meta.error}</span>
                    )}
                  </label>
                  <Input autoComplete="email" type="email" {...input} />
                </>
              )}
            </Field>
            <button type="submit">Create an account</button>
          </form>
        )}
      </Form>
    </div>
  );
};
