"use client";

import type React from "react";
import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Eye, EyeOff, Check, X } from "lucide-react";

interface PasswordRequirement {
  id: string;
  label: string;
  validate: (password: string) => boolean;
}

interface ResetPasswordFormProps {
  email: string; // now required
}

export function ResetPasswordForm({ email }: ResetPasswordFormProps) {
  const [code, setCode] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [success, setSuccess] = useState(false);
  const router = useRouter();

  const passwordRequirements: PasswordRequirement[] = [
    { id: "length", label: "At least 8 characters", validate: (pwd) => pwd.length >= 8 },
    { id: "uppercase", label: "At least one uppercase letter", validate: (pwd) => /[A-Z]/.test(pwd) },
    { id: "lowercase", label: "At least one lowercase letter", validate: (pwd) => /[a-z]/.test(pwd) },
    { id: "number", label: "At least one number", validate: (pwd) => /\d/.test(pwd) },
    { id: "special", label: "At least one special character", validate: (pwd) => /[!@#$%^&*()_+\-=[\]{};':"\\|,.<>/?]/.test(pwd) },
  ];

  const getRequirementStatus = (requirement: PasswordRequirement) =>
    requirement.validate(newPassword);

  const allRequirementsMet = passwordRequirements.every(getRequirementStatus);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!email?.trim()) {
      setError("Missing email address. Please retry the reset link.");
      return;
    }

    if (!code.trim()) {
      setError("Please enter the reset code sent to your email");
      return;
    }

    if (!allRequirementsMet) {
      setError("Password does not meet all requirements");
      return;
    }

    if (newPassword !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }

    setIsSubmitting(true);

    try {
      const res = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          email,
          code,
          new_password: newPassword,
        }),
      });

      const data = await res.json();

      if (!res.ok) throw new Error(data.error || "Failed to reset password");

      setSuccess(true);
      setTimeout(() => router.push("/auth/login"), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "An error occurred");
    } finally {
      setIsSubmitting(false);
    }
  };

  if (success) {
    return (
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl">Password Reset</CardTitle>
        </CardHeader>
        <CardContent>
          <Alert className="border-green-200 bg-green-50">
            <AlertDescription className="text-green-800">
              Your password has been reset successfully! Redirecting you to sign in...
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle className="text-2xl">Reset Password</CardTitle>
        <CardDescription>
          Enter the reset code sent to <strong>{email}</strong> and create a new password.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <label htmlFor="code" className="text-sm font-medium">
              Reset Code
            </label>
            <Input
              id="code"
              type="text"
              placeholder="Enter the code sent to your email"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              disabled={isSubmitting}
              required
            />
          </div>

          <div className="space-y-2">
            <label htmlFor="newPassword" className="text-sm font-medium">
              New Password
            </label>
            <div className="relative">
              <Input
                id="newPassword"
                type={showPassword ? "text" : "password"}
                placeholder="Create a strong password"
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                disabled={isSubmitting}
                required
              />
              <button
                type="button"
                onClick={() => setShowPassword(!showPassword)}
                disabled={isSubmitting}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
              </button>
            </div>
          </div>

          <div className="space-y-2">
            <label htmlFor="confirmPassword" className="text-sm font-medium">
              Confirm Password
            </label>
            <Input
              id="confirmPassword"
              type="password"
              placeholder="Confirm your new password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              disabled={isSubmitting}
              required
            />
          </div>

          <div className="space-y-2 p-3 bg-muted rounded-lg">
            <p className="text-xs font-semibold text-muted-foreground">
              Password requirements:
            </p>
            {passwordRequirements.map((req) => {
              const isMet = getRequirementStatus(req);
              return (
                <div key={req.id} className="flex items-center gap-2 text-xs">
                  {isMet ? (
                    <Check size={16} className="text-green-600" />
                  ) : (
                    <X size={16} className="text-muted-foreground" />
                  )}
                  <span className={isMet ? "text-green-600 font-medium" : "text-muted-foreground"}>
                    {req.label}
                  </span>
                </div>
              );
            })}
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={isSubmitting || !allRequirementsMet}
          >
            {isSubmitting ? "Resetting..." : "Reset Password"}
          </Button>
        </form>

        <div className="mt-4 text-center text-sm">
          <Link href="/auth/login" className="text-primary hover:underline font-medium">
            Back to Sign In
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
