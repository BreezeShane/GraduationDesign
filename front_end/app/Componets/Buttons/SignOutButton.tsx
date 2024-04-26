import React, { useCallback, useState } from 'react';
import { Button, Form, Input, Modal, notification } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { setAuthToken } from '@/app/Utils';
import type { SignStatusProperty } from '../NavBar';

const SignOutButton: React.FC<SignStatusProperty> = (props) => {
    const [open, setOpen] = useState(false);
    const {signStatus, changeStatus, messageClient} = props;

    const ChangeState = useCallback(() => {
      changeStatus(!signStatus)
    },[changeStatus, signStatus])
    
    const handleOk = () => {
        sessionStorage.clear();
        setAuthToken(undefined);
        messageClient.info({
            message: `Success to sign out!`,
            description: "Now you should sign in to use the insect identifier system!",
            placement: 'topLeft',
            duration: 1,
            type: 'success'
          });
        ChangeState();
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
        <Button type="primary" shape="round" icon={<LogoutOutlined />} size={'large'} onClick={showModal}>
          Sign Out
        </Button>
        <Modal title="Sign Out"
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