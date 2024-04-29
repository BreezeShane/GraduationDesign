import React, { useCallback, useState } from 'react';
import { notification } from 'antd';
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

const NavBar: React.FC<{messageClient: NotificationInstance}> = (props) => {
  const { messageClient } = props;
  const [signStatus, setSignStatus] = useState(sessionStorage.getItem('token')? true : false);
  
  const changeStatus = useCallback((signStatus: boolean) => {
    setSignStatus(signStatus);
  }, []);

  return (
    <div style={{
      display: 'flex',
      justifyContent: 'flex-end',
      alignItems: 'center',
      height: "100%"
    }}>
        <SignOutButton isVisible={signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={messageClient} />
        <SignUpButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={messageClient} />
        <SignInButton isVisible={!signStatus} signStatus={signStatus} changeStatus={changeStatus} messageClient={messageClient} />
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