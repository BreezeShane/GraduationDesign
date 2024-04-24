import React, { useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal } from 'antd';
import { PlusCircleOutlined, UserOutlined, KeyOutlined } from '@ant-design/icons';

type FieldType = {
  username?: string;
  password?: string;
  repassword?: string;
  email?: string;
};

const SignUpButton: React.FC = () => {
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
      <Button type="primary" shape="round" icon={<PlusCircleOutlined />} size={'large'} onClick={showModal}>
        Sign Up
      </Button>
      <Modal title="Title"
        open={open}
        onOk={handleOk}
        confirmLoading={confirmLoading}
        onCancel={handleCancel}
      >
        <Form
          name="sign_up_form"
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
            name="email"
            rules={[{ required: true, message: 'Please input your username!' }]}
          >
            <Input size="large" placeholder="User EMail" prefix={<UserOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="Password"
            name="password"
            rules={[{
              required: true,
              pattern:
                  /^(?![^a-zA-Z]+$)(?!\\D+$).{8,16}$/,
              message: "Need 8-16 characters containing letters and numbers",
          }]}
          >
            <Input.Password size="large" placeholder="Password" prefix={<KeyOutlined />} />
          </Form.Item>
          <Form.Item<FieldType>
            label="Confirm Password"
            name="repassword"
            dependencies={['password']}
            rules={[
              ({ getFieldValue }) => ({
                  validator(rule, value) {
                      if (!value || getFieldValue('password') === value) {
                          return Promise.resolve();
                      }
                      return Promise.reject('Please keep the same to your password above!');
                  },
              }),
          ]}
          >
            <Input.Password size="large" placeholder="Confirm Password" prefix={<KeyOutlined />} />
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

export default SignUpButton;