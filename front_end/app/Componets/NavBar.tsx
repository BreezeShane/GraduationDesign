import React, { useState } from 'react';
import type { MenuProps } from 'antd';
import { Menu } from 'antd';
import SignInButton from './FormButtons/SignInButton';
import SignUpButton from './FormButtons/SignUpButton';

const items: MenuProps['items'] = [
  {
    label: (
      <SignUpButton />
    ),
    key: 'sign_up',
  },
  {
    label: (
      <SignInButton />
    ),
    key: 'sign_in',
  },
];

const NavBar: React.FC = () => {
  const [current, setCurrent] = useState('mail');

  const onClick: MenuProps['onClick'] = (e) => {
    console.log('click ', e);
    setCurrent(e.key);
  };

  return <Menu onClick={onClick} selectedKeys={[current]} mode="horizontal" items={items} />;
};

export default NavBar;