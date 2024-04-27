import React, { useCallback, useEffect, useRef, useState } from 'react';
import type { MenuProps } from 'antd';
import { Menu, notification } from 'antd';
import SignInButton from './Buttons/SignInButton';
import SignUpButton from './Buttons/SignUpButton';
import SignOutButton from './Buttons/SignOutButton';
import { NotificationInstance } from 'antd/es/notification/interface';

export type SignStatusProperty = {
  isVisible: boolean,
  signStatus: boolean,
  changeStatus: Function,
  messageClient: NotificationInstance
}

const NavBar: React.FC = () => {
  const [api, contextHolder] = notification.useNotification();

  const [signStatus, setSignStatus] = useState(sessionStorage.getItem('token')? true : false);
  
  const changeStatus = useCallback((signStatus: boolean) => {
    setSignStatus(signStatus);
  }, []);
  // let items = [
  //   signStatus ? { key: 'sign_out', label: (<SignOutButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  //   !signStatus ? { key: 'sign_up', label: (<SignUpButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  //   !signStatus ? { key: 'sign_in', label: (<SignInButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  // ];
  // useEffect(() => {
  //   items = [
  //     signStatus ? { key: 'sign_out', label: (<SignOutButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  //     !signStatus ? { key: 'sign_up', label: (<SignUpButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  //     !signStatus ? { key: 'sign_in', label: (<SignInButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)} : null,
  //   ];
  // }, [signStatus]);
  // const items = [
  //   { key: 'sign_out', label: (<SignOutButton isVisible={signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)},
  //   { key: 'sign_up', label: (<SignUpButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)},
  //   { key: 'sign_in', label: (<SignInButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />)}
  // ];
  

  // console.log(signStatus, items);

  return (
    <div style={{
      display: 'flex',
      justifyContent: 'flex-end',
      alignItems: 'center',
      height: "100%"
    }}>
        {contextHolder}
        <SignOutButton isVisible={signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />
        <SignUpButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />
        <SignInButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={api} />
    </div>
  );

//   return (
//   <>
//     {contextHolder}
//     <Menu mode="horizontal">
//       { signStatus ? <SignOutButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
//       { !signStatus ? <SignUpButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
//       { !signStatus ? <SignInButton signStatus={signStatus} changeStatus={changeStatus} messageClient={api} /> : null }
//     </Menu>
//   </>);
};

export default NavBar;