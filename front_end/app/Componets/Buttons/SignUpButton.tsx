import React, { useCallback, useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal, Space, notification } from 'antd';
import { PlusCircleOutlined, UserOutlined, KeyOutlined, MailOutlined } from '@ant-design/icons';
import axios from 'axios';
import { NotificationInstance } from 'antd/es/notification/interface';

type FieldType = {
  username?: string;
  password?: string;
  repassword?: string;
  email?: string;
};

type SignStatusProperty = {
  isVisible: boolean,
  signStatus: boolean,
  changeStatus: Function,
  messageClient: NotificationInstance
}

const SignUpButton: React.FC<SignStatusProperty> = (props) => {
  const [open, setOpen] = useState(false);
  const [form] = Form.useForm();
  const {isVisible, signStatus, changeStatus, messageClient} = props;

  const ChangeState = useCallback(() => {
    changeStatus(!signStatus)
  },[changeStatus, signStatus])

  const showModal = () => {
    setOpen(true);
  };

  const onFinish: FormProps<FieldType>['onFinish'] = (values) => {
    axios.post('/sign_up', {
      username: values.username,
      password: values.password,
      repassword: values.repassword,
      email: values.email
    }).then(function (response) {
      messageClient.success({
        message: `Success to sign up a new account!`,
        description: "Now you can go to sign in by this account!",
        placement: 'topLeft',
        duration: 2,
      });
      ChangeState();
      setOpen(false);
    })
    .catch(function (error) {
        console.log(error);
        messageClient.success({
          message: `Failed to sign up a new account!`,
          description: "Please check your inputs!",
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
    <div style={{ width: 130, display: isVisible? "block" : "none" }}>
      <Button style={{ width: 125 }} type="primary" shape="round" icon={<PlusCircleOutlined />} size={'large'} onClick={showModal}>
        Sign Up
      </Button>
      <Modal title="Sign up"
        open={open}
        onCancel={handleCancel}
        footer={null}
      >
        <Form
          name="sign_up_form"
          form={form}
          labelCol={{ span: 8 }}
          wrapperCol={{ span: 16 }}
          style={{ maxWidth: 600 }}
          onFinish={onFinish}
          onFinishFailed={onFinishFailed}
          autoComplete="off"
        >
          <Form.Item<FieldType>
            label="User Name"
            name="username"
            rules={[{ required: true, message: 'Please input your username!' }]}
          >
            <Input size="large" placeholder="User Name" prefix={<UserOutlined />} />
          </Form.Item>

          <Form.Item<FieldType>
            label="User Email"
            name="email"
            rules={[{ required: true, message: 'Please input your Email!' }]}
          >
            <Input size="large" placeholder="User EMail" prefix={<MailOutlined />} />
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
              {
                required: true
              }
          ]}
          >
            <Input.Password size="large" placeholder="Confirm Password" prefix={<KeyOutlined />} />
          </Form.Item>


          <Form.Item wrapperCol={{ offset: 8, span: 16 }}>
            <Space>
              <Button type="primary" htmlType="submit">
                Submit
              </Button>
              <Button danger onClick={clearForm}>
                Clear
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
};

export default SignUpButton;