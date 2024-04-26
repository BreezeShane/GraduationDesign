import React, { useRef, useState } from 'react';
import type { MenuProps } from 'antd';
import { Menu } from 'antd';
import SignInButton from './FormButtons/SignInButton';
import SignUpButton from './FormButtons/SignUpButton';
import SignOutButton from './FormButtons/SignOutButton';

const items = [
  {
    key: 'sign_out',
    hidden: true,
    label: (
      <SignOutButton />
    ),
    
  },
  {
    key: 'sign_up',
    hidden: false,
    label: (
      <SignUpButton />
    ),
    
  },
  {
    key: 'sign_in',
    hidden: false,
    label: (
      <SignInButton />
    ),
    
  },
];

const NavBar: React.FC = () => {
  const [current, setCurrent] = useState('mail');
  const buttonsRef = useRef(null);
  const [status, setSignInStatus] = useState(false);

  return <Menu selectedKeys={[current]} mode="horizontal" items={items} />;
};

export default NavBar;