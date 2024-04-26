import React, { useState } from 'react';
import type { FormProps } from 'antd';
import { Button, Form, Input, Modal, notification } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { setAuthToken } from '@/app/Utils';
import axios from 'axios';

const SignOutButton: React.FC = () => {
    const [open, setOpen] = useState(false);
    const [api, contextHolder] = notification.useNotification();

    const handleOk = () => {
        sessionStorage.clear();
        setAuthToken(undefined);
        api.info({
            message: `Success to sign out!`,
            description: "Now you should sign in to use the insect identifier system!",
            placement: 'topLeft',
            duration: 2,
          });
        setOpen(false);
    }
  
    const showModal = () => {
        setOpen(true);
    };
  
    const handleCancel = () => {
        setOpen(false);
    };
  
    return (
      <>
        {contextHolder}
        <Button type="primary" shape="round" icon={<LogoutOutlined />} size={'large'} onClick={showModal}>
          Sign Out
        </Button>
        <Modal title="Caution"
          open={open}
          onOk={handleOk}
          onCancel={handleCancel}
        >
            Would you like to sign out?
        </Modal>
      </>
    );
  };
  
  export default SignOutButton;