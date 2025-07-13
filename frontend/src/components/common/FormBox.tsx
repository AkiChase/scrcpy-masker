import { Form, type FormProps } from "antd";

type FormBoxProps = React.PropsWithChildren<FormProps>;

export default function FormBox({ children, ...rest }: FormBoxProps) {
  return (
    <Form layout="vertical" {...rest}>
      {children}
    </Form>
  );
}
