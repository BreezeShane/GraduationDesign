import React, { useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal, notification } from 'antd';
import { LoginOutlined, LogoutOutlined, UserOutlined, KeyOutlined } from '@ant-design/icons';
import { POST } from '@/app/Agent';

type FieldType = {
  useremail?: string;
  password?: string;
};

const SignInButton: React.FC = () => {
  const [open, setOpen] = useState(false);
  const [confirmLoading, setConfirmLoading] = useState(false);
  const [api, contextHolder] = notification.useNotification();
  const [form] = Form.useForm();

  const showModal = () => {
    setOpen(true);
  };
  
  const onFinish: FormProps<FieldType>['onFinish'] = (values) => {
    POST('/sign_in', {
      useremail: values.useremail,
      password: values.password
    }).then(function (response) {
      console.log(response);
      api.info({
        message: `Success to sign in!`,
        description: "Now you can use the insect identifier system!",
        placement: 'topLeft',
        duration: 2,
      });
      setOpen(false);
    })
    .catch(function (error) {
        console.log(error);
        api.info({
          message: `Failed to sign in!`,
          description: "Please check your user email or password!",
          placement: 'topLeft',
          duration: 2,
        });
    });
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
      {contextHolder}
      <Button type="primary" shape="round" icon={<LoginOutlined />} size={'large'} onClick={showModal}>
        Sign In
      </Button>
      <Modal title="Title"
        open={open}
        confirmLoading={confirmLoading}
        onCancel={handleCancel}
        footer={null}
      >
        <Form
          name="sign_in_form"
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
            label="User Email"
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