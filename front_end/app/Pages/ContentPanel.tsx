import axios from 'axios';
import { Tabs, ConfigProvider } from 'antd';
import React, { ReactNode, useEffect, useState } from 'react';
import { NotificationInstance } from 'antd/es/notification/interface';

import { setAuthToken } from '../Utils';
import Common from './SubPages/Common';
import WebSSH from './SubPages/WebSSH';
import Commands from './SubPages/Commands';
import UserInfo from './SubPages/UserInfo';
import UserManage from './SubPages/UserManage';
import ModelManage from './SubPages/ModelManage';
import FeedbackManage from './SubPages/FeedbackManage';
import LabelData from './SubPages/LabelData';

const { TabPane } = Tabs;

enum Role {
  CommonUser,
  UserAdmin,
  ModelAdmin,
  SuperRoot // onDebug
}

const generateItem = (label: string, key: string, children: ReactNode) => {
  return (
    <TabPane
      tab={label}
      key={key}
      style={{ minHeight: "100vh" }}
    >
      {children}
    </TabPane>
  )
}

const checkRole = (role: string) => {
  switch(role){
    case "User Administrator": return Role.UserAdmin;
    case "Model Administrator": return Role.ModelAdmin;
    case "Common User": return Role.CommonUser;
    case "Super Root": return Role.SuperRoot;
    default: return Role.CommonUser;
  }
}

const ContentPanel: React.FC<{signStatus: boolean, messageClient: NotificationInstance}> = (props) => {
  const { signStatus, messageClient } = props;
  const [user_role, SetUserRole] = useState("");
  const [useremail, setUserEmail] = useState<string | null>();

  useEffect(() => {
    setUserEmail(sessionStorage.getItem('useremail'));
  }, [])
  let Pages: Array<ReactNode> = [];
  if (signStatus && useremail) {
    axios.get(`/user/check_role/${useremail}`)
      .then((res) => {
        SetUserRole(res.data);
      }).catch((err) => {
        if (err.response.status === 401) {
          setAuthToken(undefined);
          messageClient.error({
            message: `Your Token Expired!`,
            description: "You should sign in again!",
            placement: 'topLeft',
            duration: 2,
          });
        } else {
          messageClient.error({
            message: `Network Error!`,
            description: "You could try it again later!",
            placement: 'topLeft',
            duration: 2,
          });
        }
    });

    Pages = [
      // All User
      generateItem("Main Page", "0", <Common messageClient={messageClient} />),
      generateItem("User Info", "1", <UserInfo messageClient={messageClient} />),
      generateItem("Label Image", "2", <LabelData messageClient={messageClient} />),
    ]
    if (checkRole(user_role) == Role.UserAdmin) {
      Pages.push(
        // User Admin
        generateItem("User Manage", "3", <UserManage messageClient={messageClient} />),
        generateItem("Feedback Manage", "4", <FeedbackManage messageClient={messageClient} />),
      );
    } else if (checkRole(user_role) == Role.ModelAdmin) {
      Pages.push(
        // Model Admin
        generateItem("Model Manage", "5", <ModelManage />),
        generateItem("WebSSH", "6", <WebSSH messageClient={messageClient} />),
        generateItem("Frequent Commands", "7", <Commands messageClient={messageClient} />),
      );
    } else if (checkRole(user_role) == Role.SuperRoot) { // onDebug
      Pages.push(
        generateItem("User Manage", "3", <UserManage messageClient={messageClient} />),
        generateItem("Feedback Manage", "4", <FeedbackManage messageClient={messageClient} />),
        generateItem("Model Manage", "5", <ModelManage />),
        generateItem("WebSSH", "6", <WebSSH messageClient={messageClient} />),
        generateItem("Frequent Commands", "7", <Commands messageClient={messageClient} />),
      )
    }
  } else {
    Pages = [];
  }
  return (
    <ConfigProvider theme={{
      components: {
        Tabs: {
          cardHeight: 100,
        },
      }
    }}>
      <Tabs
        size='large'
        defaultActiveKey="0"
        tabPosition={"left"}
        style={{ height: "100%" }}
      >
        {Pages}
      </Tabs>
    </ConfigProvider>
  );
};

export default ContentPanel;