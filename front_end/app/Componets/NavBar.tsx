import React, { useCallback, useRef, useState } from 'react';
import type { MenuProps } from 'antd';
import { Menu, notification } from 'antd';
import SignInButton from './Buttons/SignInButton';
import SignUpButton from './Buttons/SignUpButton';
import SignOutButton from './Buttons/SignOutButton';
import { NotificationInstance } from 'antd/es/notification/interface';

export type SignStatusProperty = {
  signStatus: boolean,
  changeStatus: Function,
  messageClient: NotificationInstance
}

const NavBar: React.FC = () => {
  const [api, contextHolder] = notification.useNotification();

  const [signStatus, setSignStatus] = useState(false);
  const changeStatus = useCallback((signStatus: boolean) => {
    setSignStatus(signStatus);
  }, []);

  // return <Menu mode='horizontal' items={items}></Menu>

  return (
  <>
    {contextHolder}
    <Menu mode="horizontal">
      { signStatus ? <SignOutButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
      { !signStatus ? <SignUpButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
      { !signStatus ? <SignInButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
    </Menu>
  </>);
};

export default NavBar;