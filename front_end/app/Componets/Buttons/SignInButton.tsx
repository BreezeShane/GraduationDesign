import React, { useCallback, useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal, Space } from 'antd';
import { LoginOutlined, UserOutlined, KeyOutlined } from '@ant-design/icons';
import { saveSessionUserInfo, setAuthToken } from '@/app/Utils';
import axios from 'axios';
import type { SignStatusProperty } from '../NavBar';

type FieldType = {
  useremail?: string;
  password?: string;
};

const SignInButton: React.FC<SignStatusProperty> = (props) => {
  const {isVisible, signStatus, changeStatus, messageClient} = props;
  const [open, setOpen] = useState(false);
  // const [api, contextHolder] = notification.useNotification();
  const [form] = Form.useForm();

  const ChangeState = useCallback(() => {
    changeStatus(!signStatus)
  },[changeStatus, signStatus])

  const showModal = () => {
    setOpen(true);
  };

  const onFinish: FormProps<FieldType>['onFinish'] = (values) => {
    if (sessionStorage.getItem('token')) {
      messageClient.error({
        message: `Forbidden Operation!`,
        description: "You have signed in!",
        placement: 'topLeft',
        duration: 2,
      });
      setOpen(false);
      return;
    }
    axios.post('/sign_in', {
      useremail: values.useremail,
      password: values.password
    }).then(function (response) {
      setAuthToken(JSON.stringify(response));
      messageClient.success({
        message: `Success to sign in!`,
        description: "Now you can use the insect identifier system!",
        placement: 'topLeft',
        duration: 2,
      });
      saveSessionUserInfo({
        useremail: values.useremail
      })
      ChangeState();
      setOpen(false);
    })
    .catch(function (error) {
        console.log(error);
        messageClient.error({
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
    <div style={{ width: 130, display: isVisible? "block" : "none" }}>
      <Button style={{ width: 125 }} type="primary" shape="round" icon={<LoginOutlined />} size={'large'} onClick={showModal}>
        Sign In
      </Button>
      <Modal title="Sign In"
        open={open}
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

export default SignInButton;