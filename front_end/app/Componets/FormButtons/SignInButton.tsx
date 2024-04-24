import React, { useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal } from 'antd';
import { LoginOutlined, LogoutOutlined, UserOutlined, KeyOutlined } from '@ant-design/icons';

type FieldType = {
  useremail?: string;
  password?: string;
};

const SignInButton: React.FC = () => {
  const [open, setOpen] = useState(false);
  const [confirmLoading, setConfirmLoading] = useState(false);
  const [modalText, setModalText] = useState('Content of the modal');
  const [form] = Form.useForm();

  const showModal = () => {
    setOpen(true);
  };

  const handleOk = () => {
    setModalText('The modal will be closed after two seconds');
    setConfirmLoading(true);
    setTimeout(() => {
      setOpen(false);
      setConfirmLoading(false);
    }, 2000);
  };
  
  const onFinish: FormProps<FieldType>['onFinish'] = (values) => {
    console.log('Success:', values);
  };
  
  const onFinishFailed: FormProps<FieldType>['onFinishFailed'] = (errorInfo) => {
    console.log('Failed:', errorInfo);
  };

  const handleCancel = () => {
    setOpen(false);
  };

  const clearForm = () => {
    form.resetFields();
  }

  return (
    <>
      <Button type="primary" shape="round" icon={<LoginOutlined />} size={'large'} onClick={showModal}>
        Sign In
      </Button>
      <Modal title="Title"
        open={open}
        onOk={handleOk}
        confirmLoading={confirmLoading}
        onCancel={handleCancel}
      >
        <Form
          name="basic"
          form={form}
          labelCol={{ span: 8 }}
          wrapperCol={{ span: 16 }}
          style={{ maxWidth: 600 }}
          initialValues={{ remember: true }}
          onFinish={onFinish}
          onFinishFailed={onFinishFailed}
          autoComplete="off"
        >
          <Form.Item<FieldType>
            label="Useremail"
            name="useremail"
            rules={[{ required: true, message: 'Please input your username!' }]}
          >
            <Input size="large" placeholder="User EMail" prefix={<UserOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="Password"
            name="password"
            rules={[{ required: true, message: 'Please input your password!' }]}
          >
            <Input.Password size="large" placeholder="Password" prefix={<KeyOutlined />} />
          </Form.Item>

          <Form.Item wrapperCol={{ offset: 8, span: 16 }}>
            <Button type="primary" htmlType="submit">
              Submit
            </Button>
            <Button danger onClick={clearForm}>
              Clear
            </Button>
          </Form.Item>
        </Form>
      </Modal>
    </>
  );
};

export default SignInButton;