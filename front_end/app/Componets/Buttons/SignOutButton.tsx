import React, { useCallback, useState } from 'react';
import { Button, Modal } from 'antd';
import { LogoutOutlined } from '@ant-design/icons';
import { setAuthToken } from '@/app/Utils';
import type { SignStatusProperty } from '../NavBar';

const SignOutButton: React.FC<SignStatusProperty> = (props) => {
    const [open, setOpen] = useState(false);
    const {isVisible, signStatus, changeStatus, messageClient} = props;

    const ChangeState = useCallback(() => {
      changeStatus(!signStatus)
    },[changeStatus, signStatus])
    
    const handleOk = () => {
        sessionStorage.clear();
        setAuthToken(undefined);
        console.log("Message should be open!")
        messageClient.success({
            message: `Success to sign out!`,
            description: "Now you should sign in to use the insect identifier system!",
            placement: 'topLeft',
            duration: 2,
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
      <div style={{ width: 130, display: isVisible? "block" : "none" }}>
        <Button style={{ width: 125 }} type="primary" shape="round" icon={<LogoutOutlined />} size={'large'} onClick={showModal}>
          Sign Out
        </Button>
        <Modal title="Sign Out"
          open={open}
          onOk={handleOk}
          onCancel={handleCancel}
        >
            Would you like to sign out?
        </Modal>
      </div>
    );
  };
  
  export default SignOutButton;